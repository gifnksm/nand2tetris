use crate::{FuncName, Label, ModuleName};
use hasm::{
    Comp, Dest, Imm, Jump, Label as AsmLabel,
    Statement::{self, self as S},
};

#[derive(Debug, Clone)]
pub(crate) struct CodeGen<'a> {
    module_name: &'a ModuleName,
    func_name: &'a FuncName,
    command_index: usize,
    stmts: Vec<Statement>,
}

impl<'a> CodeGen<'a> {
    pub(crate) fn new(
        module_name: &'a ModuleName,
        func_name: &'a FuncName,
        command_index: usize,
    ) -> Self {
        Self {
            module_name,
            func_name,
            command_index,
            stmts: vec![],
        }
    }

    pub(crate) fn into_statements(self) -> Vec<Statement> {
        self.stmts
    }

    pub(crate) fn bootstrap(&mut self, name: &FuncName) {
        self.load_imm_d(Imm::try_new(256).unwrap());
        self.store_d_address(AsmLabel::SP);
        self.call(name, 0);
    }

    pub(crate) fn push_imm(&mut self, imm: Imm) {
        self.load_imm_d(imm);
        self.push_d();
    }

    pub(crate) fn push_dynamic_segment(&mut self, base_register: AsmLabel, index: Imm) {
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

    pub(crate) fn pop_dynamic_segment(&mut self, base_register: AsmLabel, index: Imm) {
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
        let label_true = self.make_internal_label(op, "true");
        let label_end = self.make_internal_label(op, "end");

        // D = A - D = x - y
        self.pop_d_a();
        self.stmts.push(S::c(Dest::D, Comp::AMinusD, Jump::Null));
        self.if_jump(label_true.clone(), Comp::D, jump);

        // false:
        self.load_false_d();
        self.jump(label_end.clone());

        // true:
        self.stmts.push(S::label(label_true));
        self.load_true_d();

        // end:
        self.stmts.push(S::label(label_end));
        self.push_d();
    }

    pub(crate) fn label(&mut self, label: &Label) {
        let label = self.make_label(label);
        self.stmts.push(S::label(label));
    }

    pub(crate) fn goto(&mut self, label: &Label) {
        let label = self.make_label(label);
        self.jump(label);
    }

    pub(crate) fn if_goto(&mut self, label: &Label) {
        let label = self.make_label(label);
        self.pop_d();
        self.if_jump(label, Comp::D, Jump::Ne);
    }

    pub(crate) fn function(&mut self, name: &FuncName, arity: u8) {
        let label = self.make_function_label(name);
        self.stmts.push(S::label(label));
        self.push_zeros(arity);
    }

    pub(crate) fn call(&mut self, name: &FuncName, arity: u8) {
        let function_label = self.make_function_label(name);
        let return_label = self.make_internal_label("call", "return");
        // push return-address
        self.stmts.extend([
            S::at_label(return_label.clone()),
            S::c(Dest::D, Comp::A, Jump::Null),
        ]);
        self.push_d();
        // push RAM[LCL]
        self.load_address_d(AsmLabel::LCL);
        self.push_d();
        // push RAM[ARG]
        self.load_address_d(AsmLabel::ARG);
        self.push_d();
        // push RAM[THIS]
        self.load_address_d(AsmLabel::THIS);
        self.push_d();
        // push RAM[THAT]
        self.load_address_d(AsmLabel::THAT);
        self.push_d();
        // RAM[ARG] = RAM[SP] - n - 5
        self.load_address_d(AsmLabel::SP);
        self.stmts.extend([
            S::a(Imm::from(arity + 5)),
            S::c(Dest::D, Comp::DMinusA, Jump::Null),
        ]);
        self.store_d_address(AsmLabel::ARG);
        // LCL = SP
        self.load_address_d(AsmLabel::SP);
        self.store_d_address(AsmLabel::LCL);
        // goto f
        self.jump(function_label);
        // (return-address)
        self.stmts.push(S::label(return_label));
    }

    pub(crate) fn return_(&mut self) {
        fn set(stmts: &mut Vec<Statement>, dest: AsmLabel, base: AsmLabel, n: u8) {
            // RAM[dest] = RAM[RAM[base] - n]
            stmts.extend([
                S::at_label(base),
                S::c(Dest::D, Comp::M, Jump::Null),
                S::a(Imm::from(n)),
                S::c(Dest::A, Comp::DMinusA, Jump::Null),
                S::c(Dest::D, Comp::M, Jump::Null),
                S::at_label(dest),
                S::c(Dest::M, Comp::D, Jump::Null),
            ])
        }

        // RAM[R13]: FRAME
        // RAM[R14]: RET
        // RAM[R13] = RAM[LCL]
        self.load_address_d(AsmLabel::LCL);
        self.store_d_address(AsmLabel::R13);
        // RAM[R14] = RAM[RAM[R13] - 5]
        set(&mut self.stmts, AsmLabel::R14, AsmLabel::R13, 5);
        // RAM[RAM[ARG]] = pop()
        self.pop_d();
        self.stmts.extend([
            S::at_label(AsmLabel::ARG),
            S::c(Dest::A, Comp::M, Jump::Null),
            S::c(Dest::M, Comp::D, Jump::Null),
        ]);
        // RAM[SP] = RAM[ARG] + 1
        self.stmts.extend([
            S::at_label(AsmLabel::ARG),
            S::c(Dest::D, Comp::MPlusOne, Jump::Null),
            S::at_label(AsmLabel::SP),
            S::c(Dest::M, Comp::D, Jump::Null),
        ]);
        // RAM[THAT] = RAM[RAM[R13] - 1]
        set(&mut self.stmts, AsmLabel::THAT, AsmLabel::R13, 1);
        // RAM[THIS] = RAM[RAM[R13] - 2]
        set(&mut self.stmts, AsmLabel::THIS, AsmLabel::R13, 2);
        // RAM[ARG] = RAM[RAM[R13] - 3]
        set(&mut self.stmts, AsmLabel::ARG, AsmLabel::R13, 3);
        // RAM[LCL] = RAM[RAM[R13] - 4]
        set(&mut self.stmts, AsmLabel::LCL, AsmLabel::R13, 4);
        // goto RAM[R14]
        self.stmts.extend([
            S::at_label(AsmLabel::R14),
            S::c(Dest::A, Comp::M, Jump::Null),
            S::c(Dest::Null, Comp::Zero, Jump::Eq),
        ]);
    }

    fn make_internal_label(&self, op: &str, id: &str) -> AsmLabel {
        AsmLabel::from(format!(
            "{}:{}:{}:{}:{}",
            self.module_name, self.func_name, self.command_index, op, id
        ))
    }

    fn make_static_label(&self, index: Imm) -> AsmLabel {
        AsmLabel::from(format!("{}.{}", self.module_name, index.value()))
    }

    fn make_label(&self, label: &Label) -> AsmLabel {
        AsmLabel::from(format!("{}.{}.{}", self.module_name, self.func_name, label))
    }

    fn make_function_label(&self, name: &FuncName) -> AsmLabel {
        AsmLabel::from(name.to_string())
    }

    fn set_segment_addr_to_d(&mut self, base_register: AsmLabel, index: Imm) {
        self.stmts.extend([
            S::at_label(base_register),
            S::c(Dest::D, Comp::M, Jump::Null),
            S::a(index),
            S::c(Dest::D, Comp::DPlusA, Jump::Null),
        ]);
    }

    fn set_segment_addr_to_a(&mut self, base_register: AsmLabel, index: Imm) {
        self.stmts.extend([
            S::at_label(base_register),
            S::c(Dest::D, Comp::M, Jump::Null),
            S::a(index),
            S::c(Dest::A, Comp::DPlusA, Jump::Null),
        ]);
    }

    fn load_true_d(&mut self) {
        self.stmts.push(S::c(Dest::D, Comp::MinusOne, Jump::Null));
    }

    fn load_false_d(&mut self) {
        self.stmts.push(S::c(Dest::D, Comp::Zero, Jump::Null));
    }

    fn load_imm_d(&mut self, imm: Imm) {
        self.stmts.extend([
            // D = imm
            S::a(imm),
            S::c(Dest::D, Comp::A, Jump::Null),
        ]);
    }

    fn load_address_d(&mut self, label: AsmLabel) {
        self.stmts.extend([
            // D = RAM[label]
            S::at_label(label),
            S::c(Dest::D, Comp::M, Jump::Null),
        ]);
    }

    fn load_dynamic_segment_d(&mut self, base_register: AsmLabel, index: Imm) {
        // A = RAM[base_register] + index
        self.set_segment_addr_to_a(base_register, index);
        self.stmts.extend([
            // D = M
            S::c(Dest::D, Comp::M, Jump::Null),
        ]);
    }

    fn load_fixed_segment_d(&mut self, register_index: Imm, index: Imm) {
        let addr = Imm::try_new(register_index.value() + index.value()).unwrap();
        self.stmts.extend([
            // D = RAM[addr]
            S::a(addr),
            S::c(Dest::D, Comp::M, Jump::Null),
        ]);
    }

    fn load_static_segment_d(&mut self, index: Imm) {
        let label = self.make_static_label(index);

        self.stmts.extend([
            // D = RAM[<module>.<index>]
            S::at_label(label),
            S::c(Dest::D, Comp::M, Jump::Null),
        ]);
    }

    fn store_d_dynamic_segment(&mut self, base_register: AsmLabel, index: Imm) {
        self.stmts.extend([
            // RAM[R13] = D
            S::at_label(AsmLabel::R13),
            S::c(Dest::M, Comp::D, Jump::Null),
        ]);
        // D = RAM[base_register] + index
        self.set_segment_addr_to_d(base_register, index);
        self.stmts.extend([
            // RAM[R14] = D
            S::at_label(AsmLabel::R14),
            S::c(Dest::M, Comp::D, Jump::Null),
            // D = RAM[R13]
            S::at_label(AsmLabel::R13),
            S::c(Dest::D, Comp::M, Jump::Null),
            // A = RAM[R14]
            S::at_label(AsmLabel::R14),
            S::c(Dest::A, Comp::M, Jump::Null),
            // M = D
            S::c(Dest::M, Comp::D, Jump::Null),
        ]);
    }

    fn store_d_fixed_segment(&mut self, register_index: Imm, index: Imm) {
        let addr = Imm::try_new(register_index.value() + index.value()).unwrap();
        self.stmts.extend([
            // RAM[addr] = D
            S::a(addr),
            S::c(Dest::M, Comp::D, Jump::Null),
        ]);
    }

    fn store_d_static_segment(&mut self, index: Imm) {
        let label = self.make_static_label(index);

        self.stmts.extend([
            // RAM[<module>.<index>] = D
            S::at_label(label),
            S::c(Dest::M, Comp::D, Jump::Null),
        ]);
    }

    fn store_d_address(&mut self, label: AsmLabel) {
        self.stmts.extend([
            // RAM[label] = D
            S::at_label(label),
            S::c(Dest::M, Comp::D, Jump::Null),
        ]);
    }

    fn push_d(&mut self) {
        self.stmts.extend([
            // RAM[RAM[SP]] = D
            S::at_label(AsmLabel::SP),
            S::c(Dest::A, Comp::M, Jump::Null),
            S::c(Dest::M, Comp::D, Jump::Null),
            // RAM[SP] = RAM[SP] + 1
            S::at_label(AsmLabel::SP),
            S::c(Dest::M, Comp::MPlusOne, Jump::Null),
        ]);
    }

    fn push_zeros(&mut self, count: u8) {
        self.stmts.push(S::at_label(AsmLabel::SP));
        for _ in 0..count {
            self.stmts.extend([
                // RAM[RAM[SP]] = 0
                S::c(Dest::A, Comp::M, Jump::Null),
                S::c(Dest::M, Comp::Zero, Jump::Null),
                // RAM[SP] = RAM[SP] + 1
                S::at_label(AsmLabel::SP),
                S::c(Dest::M, Comp::MPlusOne, Jump::Null),
            ]);
        }
    }

    fn pop_d(&mut self) {
        self.stmts.extend([
            // RAM[SP] = RAM[SP] - 1
            S::at_label(AsmLabel::SP),
            S::c(Dest::M, Comp::MMinusOne, Jump::Null),
            // D = RAM[RAM[SP]]
            S::c(Dest::A, Comp::M, Jump::Null),
            S::c(Dest::D, Comp::M, Jump::Null),
        ]);
    }

    fn pop_a(&mut self) {
        self.stmts.extend([
            // RAM[SP] = RAM[SP] - 1
            S::at_label(AsmLabel::SP),
            S::c(Dest::M, Comp::MMinusOne, Jump::Null),
            // A = RAM[RAM[SP]]
            S::c(Dest::A, Comp::M, Jump::Null),
            S::c(Dest::A, Comp::M, Jump::Null),
        ]);
    }

    fn pop_d_a(&mut self) {
        self.pop_d();
        self.pop_a();
    }

    fn jump(&mut self, label: AsmLabel) {
        self.stmts.push(S::AtLabel(label));
        self.stmts.push(S::c(Dest::Null, Comp::Zero, Jump::Eq));
    }

    fn if_jump(&mut self, label: AsmLabel, comp: Comp, jump: Jump) {
        self.stmts.extend([
            // if (D) jump label
            S::at_label(label),
            S::c(Dest::Null, comp, jump),
        ]);
    }
}
