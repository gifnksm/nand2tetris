use super::Executable;
use crate::Statement;

impl Executable {
    pub fn disassemble(exec: hack::Executable) -> Self {
        let stmts = exec
            .instructions()
            .iter()
            .map(|inst| Statement::disassemble(inst))
            .collect();
        Self { stmts }
    }
}
