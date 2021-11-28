pub use self::parser::*;
use crate::Instruction;

mod parser;

#[derive(Debug, Clone)]
pub struct Executable {
    insts: Vec<Instruction>,
}

impl Executable {
    pub fn instructions(&self) -> &[Instruction] {
        &self.insts
    }
}
