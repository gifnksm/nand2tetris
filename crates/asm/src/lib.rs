use std::io::BufRead;
use std::path::PathBuf;

pub use error::*;
pub use inst::*;

mod error;
mod inst;
mod parser;
mod stmt;
mod sym_tab;

pub fn parse<R>(filename: impl Into<PathBuf>, reader: R) -> Result<Vec<Instruction>>
where
    R: BufRead,
{
    let stmts = parser::Parser::new(filename, reader).parse()?;
    let sym_tab = sym_tab::from_stmts(&stmts)?;
    let insts = stmts
        .into_iter()
        .filter_map(|stmt| stmt.kind().to_inst(&sym_tab))
        .collect();
    Ok(insts)
}
