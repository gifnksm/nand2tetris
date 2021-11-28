use super::{Label, Statement};
use hack::{Comp, Dest, InstC, Jump};
use hack::{Imm, Instruction};
use std::collections::HashMap;

impl Statement {
    pub(crate) fn assemble(&self, symbols: &HashMap<Label, u16>, insts: &mut Vec<Instruction>) {
        match self {
            Statement::Label(_) => {}
            Statement::AtLabel(label) => {
                let imm = *symbols.get(label).unwrap();
                assemble_a(imm, insts);
            }
            Statement::C(c) => insts.push(Instruction::C(*c)),
            Statement::A(a) => assemble_a(*a, insts),
        }
    }
}

fn assemble_a(a: u16, insts: &mut Vec<Instruction>) {
    if a <= Imm::MAX.value() {
        insts.push(Instruction::A(Imm::try_new(a).unwrap()));
    } else {
        let not_a = !a;
        insts.push(Instruction::A(Imm::try_new(not_a).unwrap()));
        insts.push(Instruction::C(InstC::new(Dest::A, Comp::NotA, Jump::Null)));
    }
}
