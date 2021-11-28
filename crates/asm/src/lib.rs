use std::io::BufRead;

pub use error::*;
pub use inst::*;
pub use stmt::*;

mod error;
mod inst;
mod parser;
mod stmt;
mod sym_tab;

pub fn parse_asm<R>(reader: R) -> Result<Vec<Instruction>>
where
    R: BufRead,
{
    let stmts = parser::parse(reader)?;
    let sym_tab = sym_tab::from_stmts(&stmts)?;
    let mut insts = vec![];
    for stmt in stmts.into_iter() {
        stmt.data().translate(&sym_tab, &mut insts);
    }
    Ok(insts)
}
