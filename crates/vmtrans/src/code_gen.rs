use hasm::{
    Comp, Dest, Imm, Jump, Label,
    Statement::{self, self as S},
};

#[derive(Debug, Clone)]
pub(crate) struct CodeGen<'a> {
    module_name: &'a str,
    command_index: usize,
    stmts: Vec<Statement>,
}

impl<'a> CodeGen<'a> {
    pub(crate) fn new(module_name: &'a str, command_index: usize) -> Self {
        Self {
            module_name,
            command_index,
            stmts: vec![],
        }
    }

    pub(crate) fn into_statements(self) -> Vec<Statement> {
        self.stmts
    }

    pub(crate) fn internal_label(&self, op: &str, id: &str) -> Label {
        Label::from(format!(
            "{}${}${}${}",
            self.module_name, self.command_index, op, id
        ))
    }

    pub(crate) fn static_label(&self, index: Imm) -> Label {
        Label::from(format!("{}.{}", self.module_name, index.value()))
    }

    pub(crate) fn set_segment_addr_to_d(&mut self, base_register: Label, index: Imm) {
        self.stmts.extend([
            S::at_label(base_register),
            S::c(Dest::D, Comp::M, Jump::Null),
            S::a(index),
            S::c(Dest::D, Comp::DPlusA, Jump::Null),
        ]);
    }

    pub(crate) fn set_segment_addr_to_a(&mut self, base_register: Label, index: Imm) {
        self.stmts.extend([
            S::at_label(base_register),
            S::c(Dest::D, Comp::M, Jump::Null),
            S::a(index),
            S::c(Dest::A, Comp::DPlusA, Jump::Null),
        ]);
    }

    pub(crate) fn load_true_d(&mut self) {
        self.stmts.push(S::c(Dest::D, Comp::MinusOne, Jump::Null));
    }

    pub(crate) fn load_false_d(&mut self) {
        self.stmts.push(S::c(Dest::D, Comp::Zero, Jump::Null));
    }

    pub(crate) fn load_imm_d(&mut self, imm: Imm) {
        self.stmts.extend([
            // D = imm
            S::a(imm),
            S::c(Dest::D, Comp::A, Jump::Null),
        ]);
    }

    pub(crate) fn load_dynamic_segment_d(&mut self, base_register: Label, index: Imm) {
        // A = RAM[base_register] + index
        self.set_segment_addr_to_a(base_register, index);
        self.stmts.extend([
            // D = M
            S::c(Dest::D, Comp::M, Jump::Null),
        ]);
    }

    pub(crate) fn load_fixed_segment_d(&mut self, register_index: Imm, index: Imm) {
        let addr = Imm::try_new(register_index.value() + index.value()).unwrap();
        self.stmts.extend([
            // D = RAM[addr]
            S::a(addr),
            S::c(Dest::D, Comp::M, Jump::Null),
        ]);
    }

    pub(crate) fn load_static_segment_d(&mut self, index: Imm) {
        let label = self.static_label(index);
        self.stmts.extend([
            // D = RAM[<module>.<index>]
            S::at_label(label),
            S::c(Dest::D, Comp::M, Jump::Null),
        ]);
    }

    pub(crate) fn store_d_dynamic_segment(&mut self, base_register: Label, index: Imm) {
        self.stmts.extend([
            // RAM[R13] = D
            S::at_label(Label::R13),
            S::c(Dest::M, Comp::D, Jump::Null),
        ]);
        // D = RAM[base_register] + index
        self.set_segment_addr_to_d(base_register, index);
        self.stmts.extend([
            // RAM[R14] = D
            S::at_label(Label::R14),
            S::c(Dest::M, Comp::D, Jump::Null),
            // D = RAM[R13]
            S::at_label(Label::R13),
            S::c(Dest::D, Comp::M, Jump::Null),
            // A = RAM[R14]
            S::at_label(Label::R14),
            S::c(Dest::A, Comp::M, Jump::Null),
            // M = D
            S::c(Dest::M, Comp::D, Jump::Null),
        ]);
    }

    pub(crate) fn store_d_fixed_segment(&mut self, register_index: Imm, index: Imm) {
        let addr = Imm::try_new(register_index.value() + index.value()).unwrap();
        self.stmts.extend([
            // RAM[addr] = D
            S::a(addr),
            S::c(Dest::M, Comp::D, Jump::Null),
        ]);
    }

    pub(crate) fn store_d_static_segment(&mut self, index: Imm) {
        let label = self.static_label(index);
        self.stmts.extend([
            // RAM[<module>.<index>] = D
            S::at_label(label),
            S::c(Dest::M, Comp::D, Jump::Null),
        ]);
    }

    pub(crate) fn push_d(&mut self) {
        self.stmts.extend([
            // RAM[RAM[SP]] = D
            S::at_label(Label::SP),
            S::c(Dest::A, Comp::M, Jump::Null),
            S::c(Dest::M, Comp::D, Jump::Null),
            // RAM[SP] = RAM[SP] - 1
            S::at_label(Label::SP),
            S::c(Dest::M, Comp::MPlusOne, Jump::Null),
        ]);
    }

    pub(crate) fn push_imm(&mut self, imm: Imm) {
        self.load_imm_d(imm);
        self.push_d();
    }

    pub(crate) fn push_dynamic_segment(&mut self, base_register: Label, index: Imm) {
        self.load_dynamic_segment_d(base_register, index);
        self.push_d();
    }

    pub(crate) fn push_fixed_segment(&mut self, register_index: Imm, index: Imm) {
        self.load_fixed_segment_d(register_index, index);
        self.push_d();
    }

    pub(crate) fn push_static_segment(&mut self, index: Imm) {
        self.load_static_segment_d(index);
        self.push_d();
    }

    pub(crate) fn pop_d(&mut self) {
        self.stmts.extend([
            // RAM[SP] = RAM[SP] - 1
            S::at_label(Label::SP),
            S::c(Dest::M, Comp::MMinusOne, Jump::Null),
            // D = RAM[RAM[SP]]
            S::c(Dest::A, Comp::M, Jump::Null),
            S::c(Dest::D, Comp::M, Jump::Null),
        ]);
    }

    pub(crate) fn pop_a(&mut self) {
        self.stmts.extend([
            // RAM[SP] = RAM[SP] - 1
            S::at_label(Label::SP),
            S::c(Dest::M, Comp::MMinusOne, Jump::Null),
            // A = RAM[RAM[SP]]
            S::c(Dest::A, Comp::M, Jump::Null),
            S::c(Dest::A, Comp::M, Jump::Null),
        ]);
    }

    pub(crate) fn pop_dynamic_segment(&mut self, base_register: Label, index: Imm) {
        self.pop_d();
        self.store_d_dynamic_segment(base_register, index);
    }

    pub(crate) fn pop_fixed_segment(&mut self, register_index: Imm, index: Imm) {
        self.pop_d();
        self.store_d_fixed_segment(register_index, index);
    }

    pub(crate) fn pop_static_segment(&mut self, index: Imm) {
        self.pop_d();
        self.store_d_static_segment(index);
    }

    pub(crate) fn pop_d_a(&mut self) {
        self.pop_d();
        self.pop_a();
    }

    pub(crate) fn goto(&mut self, label: Label) {
        self.stmts.push(S::AtLabel(label));
        self.stmts.push(S::c(Dest::Null, Comp::Zero, Jump::Eq));
    }

    pub(crate) fn goto_if(&mut self, label: Label, comp: Comp, jump: Jump) {
        self.stmts.extend([
            // if (D) goto label
            S::at_label(label),
            S::c(Dest::Null, comp, jump),
        ]);
    }

    pub(crate) fn unary_op(&mut self, comp: Comp) {
        self.pop_d();
        self.stmts.push(S::c(Dest::D, comp, Jump::Null));
        self.push_d();
    }

    pub(crate) fn binary_op(&mut self, comp: Comp) {
        self.pop_d_a();
        self.stmts.push(S::c(Dest::D, comp, Jump::Null));
        self.push_d();
    }

    pub(crate) fn cond(&mut self, op: &str, jump: Jump) {
        let label_true = self.internal_label(op, "true");
        let label_end = self.internal_label(op, "end");

        // D = A - D = x - y
        self.pop_d_a();
        self.stmts.push(S::c(Dest::D, Comp::AMinusD, Jump::Null));
        self.goto_if(label_true.clone(), Comp::D, jump);

        // false:
        self.load_false_d();
        self.goto(label_end.clone());

        // true:
        self.stmts.push(S::label(label_true));
        self.load_true_d();

        // end:
        self.stmts.push(S::label(label_end));
        self.push_d();
    }
}
