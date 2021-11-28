use super::{Command, FuncName, Segment};
use crate::{code_gen::CodeGen, ModuleName};
use asm::{Label as AsmLabel, Statement};
use hack::{Comp, Imm, Jump};

impl Command {
    pub(crate) fn translate(
        &self,
        module_name: &ModuleName,
        func_name: &FuncName,
        index: usize,
        stmts: &mut Vec<Statement>,
    ) {
        let mut gen = CodeGen::new(module_name, func_name, index, stmts);
        match self {
            Command::Add => gen.binary_op(Comp::DPlusA),
            Command::Sub => gen.binary_op(Comp::AMinusD),
            Command::Neg => gen.unary_op(Comp::MinusD),
            Command::Eq => gen.cond("eq", Jump::Eq),
            Command::Gt => gen.cond("gt", Jump::Gt),
            Command::Lt => gen.cond("lt", Jump::Lt),
            Command::And => gen.binary_op(Comp::DAndA),
            Command::Or => gen.binary_op(Comp::DOrA),
            Command::Not => gen.unary_op(Comp::NotD),
            Command::Push(Segment::Local, index) => gen.push_dynamic_segment(AsmLabel::LCL, *index),
            Command::Push(Segment::Argument, index) => {
                gen.push_dynamic_segment(AsmLabel::ARG, *index)
            }
            Command::Push(Segment::This, index) => gen.push_dynamic_segment(AsmLabel::THIS, *index),
            Command::Push(Segment::That, index) => gen.push_dynamic_segment(AsmLabel::THAT, *index),
            Command::Push(Segment::Pointer, index) => gen.push_fixed_segment(Imm::THIS, *index),
            Command::Push(Segment::Temp, index) => gen.push_fixed_segment(Imm::R5, *index),
            Command::Push(Segment::Static, index) => gen.push_static_segment(*index),
            Command::Push(Segment::Constant, imm) => gen.push_imm(*imm),
            Command::Pop(Segment::Local, index) => gen.pop_dynamic_segment(AsmLabel::LCL, *index),
            Command::Pop(Segment::Argument, index) => {
                gen.pop_dynamic_segment(AsmLabel::ARG, *index)
            }
            Command::Pop(Segment::This, index) => gen.pop_dynamic_segment(AsmLabel::THIS, *index),
            Command::Pop(Segment::That, index) => gen.pop_dynamic_segment(AsmLabel::THAT, *index),
            Command::Pop(Segment::Pointer, index) => gen.pop_fixed_segment(Imm::THIS, *index),
            Command::Pop(Segment::Temp, index) => gen.pop_fixed_segment(Imm::R5, *index),
            Command::Pop(Segment::Static, index) => gen.pop_static_segment(*index),
            Command::Pop(_, _) => unreachable!("{:?}", self),
            Command::Label(label) => gen.label(label),
            Command::Goto(label) => gen.goto(label),
            Command::IfGoto(label) => gen.if_goto(label),
            Command::Function(name, arity) => gen.function(name, *arity),
            Command::Call(name, arity) => gen.call(name, *arity),
            Command::Return => gen.return_(),
        }
    }
}
