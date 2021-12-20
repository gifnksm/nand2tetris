use self::emitter::Emitter;
use super::{BasicBlock, BbId, CfgClass, CfgStatement, CfgSubroutine, Exit};
use crate::{
    ast::{BinaryOp, SubroutineKind, UnaryOp},
    symbol_table::VarSymbol,
    token::WithLoc,
    typed_ast::{
        TypedDoStatement, TypedExpression, TypedLetStatement, TypedSubroutineCall, TypedTerm,
    },
};
use std::{fmt, iter};
use vm::{Command, Segment};

mod emitter;

impl WithLoc<CfgClass> {
    pub fn to_vm(&self) -> Vec<Command> {
        let mut e = Emitter::new();
        self.emit_vm(&self.data, &mut e);
        e.into_commands()
    }
}

trait EmitVm {
    fn emit_vm(&self, class: &CfgClass, e: &mut Emitter);
}

trait EmitVmSub {
    fn emit_vm(&self, class: &CfgClass, sub: &CfgSubroutine, e: &mut Emitter);
}

#[derive(Debug)]
struct EmitVmBbParam<'a> {
    class: &'a CfgClass,
    sub: &'a CfgSubroutine,
    prev_bb: Option<BbId>,
    next_bb: Option<BbId>,
}

trait EmitVmBb {
    fn emit_vm(&self, p: &EmitVmBbParam, e: &mut Emitter);
}

impl<T> EmitVm for WithLoc<T>
where
    T: EmitVm,
{
    fn emit_vm(&self, class: &CfgClass, e: &mut Emitter) {
        self.data.emit_vm(class, e)
    }
}

impl<T> EmitVmSub for WithLoc<T>
where
    T: EmitVmSub,
{
    fn emit_vm(&self, class: &CfgClass, sub: &CfgSubroutine, e: &mut Emitter) {
        self.data.emit_vm(class, sub, e)
    }
}

impl<T> EmitVmBb for WithLoc<T>
where
    T: EmitVmBb,
{
    fn emit_vm(&self, p: &EmitVmBbParam, e: &mut Emitter) {
        self.data.emit_vm(p, e)
    }
}

impl EmitVm for CfgClass {
    fn emit_vm(&self, class: &CfgClass, e: &mut Emitter) {
        for sub in &self.subs {
            sub.emit_vm(class, &sub.data, e);
        }
    }
}

impl EmitVmSub for CfgSubroutine {
    fn emit_vm(&self, class: &CfgClass, sub: &CfgSubroutine, e: &mut Emitter) {
        e.emit_function(&class.name.data, &self.name.data, self.vars.len());

        match self.kind.data {
            SubroutineKind::Constructor => {
                e.emit_push_constant(class.fields.len());
                e.emit_call("Memory", "alloc", 1);
                e.emit_pop_this_addr();
            }
            SubroutineKind::Function => {}
            SubroutineKind::Method => {
                e.emit_push(Segment::Argument, 0);
                e.emit_pop_this_addr();
            }
        }

        let prev_bbs = Iterator::chain(
            iter::once(None),
            self.blocks.iter().map(|b| Some(b.data.id)),
        );
        let next_bbs = Iterator::chain(
            self.blocks.iter().map(|b| Some(b.data.id)).skip(1),
            iter::repeat(None),
        );
        for (block, (prev_bb, next_bb)) in self.blocks.iter().zip(prev_bbs.zip(next_bbs)) {
            let p = EmitVmBbParam {
                class,
                sub,
                prev_bb,
                next_bb,
            };
            block.emit_vm(&p, e);
        }
    }
}

impl EmitVmBb for BasicBlock {
    fn emit_vm(&self, p: &EmitVmBbParam, e: &mut Emitter) {
        let Self {
            id,
            src_ids,
            stmts,
            exit,
        } = self;

        let is_entry_block = p.sub.entry_id == *id;
        let is_src_prev = matches!(src_ids.as_slice(), &[src_id] if Some(src_id) == p.prev_bb) && {
            let prev_bb = p.prev_bb.unwrap();
            let prev_bb_index = p.sub.block_index_map[&prev_bb];
            let prev_exit = &p.sub.blocks[prev_bb_index].data.exit;
            matches!(prev_exit, Exit::Goto(_))
                || matches!(prev_exit, Exit::If(_, _, else_id) if *else_id == *id)
        };
        if !is_entry_block && !is_src_prev {
            e.emit_label(*id);
        }
        for stmt in stmts {
            stmt.emit_vm(p, e);
        }
        exit.emit_vm(p, e)
    }
}

impl EmitVmBb for CfgStatement {
    fn emit_vm(&self, p: &EmitVmBbParam, e: &mut Emitter) {
        match self {
            Self::Let(stmt) => stmt.emit_vm(p, e),
            Self::Do(stmt) => stmt.emit_vm(p, e),
        }
    }
}

impl EmitVmBb for TypedLetStatement {
    fn emit_vm(&self, p: &EmitVmBbParam, e: &mut Emitter) {
        let Self {
            target,
            target_index,
            expr,
        } = self;
        expr.emit_vm(p, e);
        let (seg, slot) = target.segment_slot();
        if let Some(target_index) = target_index {
            target_index.emit_vm(p, e);
            e.emit_push(seg, slot);
            e.emit_add();
            e.emit_pop_that_addr();
            e.emit_pop(Segment::That, 0);
        } else {
            e.emit_pop(seg, slot);
        }
    }
}

impl EmitVmBb for TypedDoStatement {
    fn emit_vm(&self, p: &EmitVmBbParam, e: &mut Emitter) {
        let Self { sub_call } = self;
        sub_call.emit_vm(p, e);
        e.emit_pop(Segment::Temp, 0);
    }
}

impl EmitVmBb for Exit {
    fn emit_vm(&self, p: &EmitVmBbParam, e: &mut Emitter) {
        match self {
            Self::Return(Some(value)) => {
                value.emit_vm(p, e);
                e.emit_return();
            }
            Self::Return(None) => {
                e.emit_push_constant(0);
                e.emit_return();
            }
            Self::Goto(label) => {
                if p.next_bb != Some(*label) {
                    e.emit_goto(*label);
                }
            }
            Self::If(cond, then_label, else_label) => {
                cond.emit_vm(p, e);
                e.emit_if_goto(*then_label);
                if p.next_bb != Some(*else_label) {
                    e.emit_goto(*else_label);
                }
            }
            Self::Unreachable => e.emit_call("Sys", "halt", 0),
        }
    }
}

impl EmitVmBb for TypedExpression {
    fn emit_vm(&self, p: &EmitVmBbParam, e: &mut Emitter) {
        self.term.emit_vm(p, e);
    }
}

impl EmitVmBb for TypedTerm {
    fn emit_vm(&self, p: &EmitVmBbParam, e: &mut Emitter) {
        match self {
            Self::Int(n) => e.emit_push_constant(n.data),
            Self::String(s) => {
                let wbytes = s.data.encode_utf16().collect::<Vec<_>>();
                e.emit_push_constant(wbytes.len());
                e.emit_call("String", "new", 1);
                for wbyte in wbytes {
                    e.emit_push_constant(wbyte);
                    e.emit_call("String", "appendChar", 2);
                }
            }
            Self::Bool(b) => {
                if b.data {
                    e.emit_push_constant(0);
                    e.emit_not();
                } else {
                    e.emit_push_constant(0);
                }
            }
            Self::Null => e.emit_push_constant(0),
            Self::This => e.emit_push_this_addr(),
            Self::Var(sym) => sym.emit_vm(p, e),
            Self::Index(sym, target_index) => {
                let (seg, slot) = sym.segment_slot();
                target_index.emit_vm(p, e);
                e.emit_push(seg, slot);
                e.emit_add();
                e.emit_pop_that_addr();
                e.emit_push_that_value();
            }
            Self::SubroutineCall(sub_call) => sub_call.emit_vm(p, e),
            Self::UnaryOp(op, operand) => {
                operand.emit_vm(p, e);
                match op.data {
                    UnaryOp::Neg => e.emit_neg(),
                    UnaryOp::Not => e.emit_not(),
                }
            }
            Self::BinaryOp(op, lhs, rhs) => {
                lhs.emit_vm(p, e);
                rhs.emit_vm(p, e);
                match op.data {
                    BinaryOp::Add => e.emit_add(),
                    BinaryOp::Sub => e.emit_sub(),
                    BinaryOp::Mul => e.emit_call("Math", "multiply", 2),
                    BinaryOp::Div => e.emit_call("Math", "divide", 2),
                    BinaryOp::And => e.emit_and(),
                    BinaryOp::Or => e.emit_or(),
                    BinaryOp::Lt => e.emit_lt(),
                    BinaryOp::Gt => e.emit_gt(),
                    BinaryOp::Eq => e.emit_eq(),
                }
            }
        }
    }
}

impl EmitVmBb for TypedSubroutineCall {
    fn emit_vm(&self, p: &EmitVmBbParam, e: &mut Emitter) {
        match self {
            TypedSubroutineCall::Method(receiver, sub_name, args) => {
                let class_name = match receiver {
                    Some(receiver) => {
                        receiver.emit_vm(p, e);
                        receiver.ty().data.to_class().unwrap()
                    }
                    None => {
                        TypedTerm::This.emit_vm(p, e);
                        &p.class.name.data
                    }
                };
                e.emit_call_with_args(class_name, &sub_name.data, true, args, p);
            }
            TypedSubroutineCall::Function(class_name, sub_name, args)
            | TypedSubroutineCall::Constructor(class_name, sub_name, args) => {
                let class_name = match class_name {
                    Some(class_name) => &class_name.data,
                    None => &p.class.name.data,
                };
                e.emit_call_with_args(class_name, &sub_name.data, false, args, p);
            }
        }
    }
}

impl EmitVmBb for VarSymbol {
    fn emit_vm(&self, _p: &EmitVmBbParam, e: &mut Emitter) {
        let (seg, slot) = self.segment_slot();
        e.emit_push(seg, slot);
    }
}
