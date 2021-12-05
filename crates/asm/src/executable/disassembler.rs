use super::Executable;
use crate::Statement;

impl Executable {
    pub fn disassemble(exec: hack::Executable) -> Self {
        let stmts = exec
            .instructions()
            .iter()
            .map(Statement::disassemble)
            .collect();
        Self { stmts }
    }
}
