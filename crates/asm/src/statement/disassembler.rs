use super::Statement;
use hack::Instruction;

impl Statement {
    pub fn disassemble(inst: &Instruction) -> Self {
        match inst {
            Instruction::A(a) => Self::A(a.value()),
            Instruction::C(c) => Self::C(*c),
        }
    }
}
