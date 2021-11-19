use std::collections::HashMap;

use crate::{
    stmt::{Statement, StatementKind},
    Error, ErrorKind, Imm,
};
use indexmap::{map::Entry, IndexMap};

#[derive(Debug, Clone)]
enum Symbol {
    Imm(Imm),
    Undefined { line: u32 },
}

impl Symbol {
    fn update(&mut self, line: u32, name: &str, imm: Imm) -> Result<(), Error> {
        match self {
            Symbol::Undefined { .. } => {
                *self = Symbol::Imm(imm);
                Ok(())
            }
            Symbol::Imm(_) => Err(Error::new(
                line,
                ErrorKind::DuplicateLabel(name.to_string()),
            )),
        }
    }
}

pub(crate) fn from_stmts(stmts: &[Statement]) -> Result<HashMap<String, Imm>, Error> {
    let mut symbol_map = predefined_map();
    insert_symbols(&mut symbol_map, stmts)?;
    let map = create_map(symbol_map)?;
    Ok(map)
}

fn predefined_map() -> IndexMap<String, Symbol> {
    let predefined = &[
        ("SP", 0),
        ("LCL", 1),
        ("ARG", 2),
        ("THIS", 3),
        ("THAT", 4),
        ("R0", 0),
        ("R1", 1),
        ("R2", 2),
        ("R3", 3),
        ("R4", 4),
        ("R5", 5),
        ("R6", 6),
        ("R7", 7),
        ("R8", 8),
        ("R9", 9),
        ("R10", 10),
        ("R11", 11),
        ("R12", 12),
        ("R13", 13),
        ("R14", 14),
        ("R15", 15),
        ("SCREEN", 0x4000),
        ("KBD", 0x6000),
    ];
    let mut map = IndexMap::new();
    for (name, value) in predefined {
        map.insert(name.to_string(), Symbol::Imm(Imm::try_new(*value).unwrap()));
    }
    map
}

fn insert_symbols(map: &mut IndexMap<String, Symbol>, stmts: &[Statement]) -> Result<(), Error> {
    let mut inst_count = 0;
    for stmt in stmts {
        let line = stmt.line();
        let kind = stmt.kind();
        if kind.is_inst() {
            inst_count += 1;
        }

        match kind {
            StatementKind::Label(name) => {
                let imm = Imm::try_new(inst_count).ok_or_else(|| {
                    Error::new(stmt.line(), ErrorKind::TooFarLabel(name.to_string()))
                })?;
                match map.entry(name.clone()) {
                    Entry::Occupied(mut e) => e.get_mut().update(stmt.line(), name, imm)?,
                    Entry::Vacant(e) => {
                        let _ = e.insert(Symbol::Imm(imm));
                    }
                }
            }
            StatementKind::AtLabel(name) => {
                map.entry(name.clone())
                    .or_insert(Symbol::Undefined { line });
            }
            StatementKind::A(_) | StatementKind::C(_) => {}
        }
    }

    Ok(())
}

fn create_map(map: IndexMap<String, Symbol>) -> Result<HashMap<String, Imm>, Error> {
    let mut result = HashMap::new();
    let mut imm_value = 0x0010;
    for (name, symbol) in map {
        match symbol {
            Symbol::Imm(imm) => {
                result.insert(name, imm);
            }
            Symbol::Undefined { line } => {
                let imm = Imm::try_new(imm_value)
                    .ok_or_else(|| Error::new(line, ErrorKind::TooManySymbols(name.to_string())))?;
                imm_value += 1;
                result.insert(name, imm);
            }
        }
    }
    Ok(result)
}
