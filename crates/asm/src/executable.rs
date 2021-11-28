pub use self::{assembler::*, parser::*};
use crate::Statement;

mod assembler;
mod disassembler;
mod parser;

#[derive(Debug, Clone)]
pub struct Executable {
    stmts: Vec<Statement>,
}

impl Executable {
    pub fn statements(&self) -> &[Statement] {
        &self.stmts
    }
}
