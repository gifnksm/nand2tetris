use crate::{Error, ErrorKind, Imm, Label, Statement, StatementWithLine};
use indexmap::{map::Entry, IndexMap};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum Symbol {
    Imm(Imm),
    Undefined { line: u32 },
}

impl Symbol {
    fn update(&mut self, line: u32, name: &Label, imm: Imm) -> Result<(), Error> {
        match self {
            Symbol::Undefined { .. } => {
                *self = Symbol::Imm(imm);
                Ok(())
            }
            Symbol::Imm(_) => Err(Error::new(line, ErrorKind::DuplicateLabel(name.clone()))),
        }
    }
}

pub(crate) fn from_stmts(stmts: &[StatementWithLine]) -> Result<HashMap<String, Imm>, Error> {
    let mut symbol_map = predefined_map();
    insert_symbols(&mut symbol_map, stmts)?;
    let map = create_map(symbol_map)?;
    Ok(map)
}

fn predefined_map() -> IndexMap<String, Symbol> {
    let predefined = &[
        (Label::SP, Imm::SP),
        (Label::LCL, Imm::LCL),
        (Label::ARG, Imm::ARG),
        (Label::THIS, Imm::THIS),
        (Label::THAT, Imm::THAT),
        (Label::R0, Imm::R0),
        (Label::R1, Imm::R1),
        (Label::R2, Imm::R2),
        (Label::R3, Imm::R3),
        (Label::R4, Imm::R4),
        (Label::R5, Imm::R5),
        (Label::R6, Imm::R6),
        (Label::R7, Imm::R7),
        (Label::R8, Imm::R8),
        (Label::R9, Imm::R9),
        (Label::R10, Imm::R10),
        (Label::R11, Imm::R11),
        (Label::R12, Imm::R12),
        (Label::R13, Imm::R13),
        (Label::R14, Imm::R14),
        (Label::R15, Imm::R15),
        (Label::SCREEN, Imm::SCREEN),
        (Label::KBD, Imm::KBD),
    ];
    let mut map = IndexMap::new();
    for (name, value) in predefined {
        map.insert(name.to_string(), Symbol::Imm(*value));
    }
    map
}

fn insert_symbols(
    map: &mut IndexMap<String, Symbol>,
    stmts: &[StatementWithLine],
) -> Result<(), Error> {
    let mut inst_count = 0;
    for stmt in stmts {
        let line = stmt.line();
        let kind = stmt.data();
        if kind.is_inst() {
            inst_count += 1;
        }

        match kind {
            Statement::Label(name) => {
                let imm = Imm::try_new(inst_count).ok_or_else(|| {
                    Error::new(stmt.line(), ErrorKind::TooFarLabel(name.to_string()))
                })?;
                match map.entry(name.to_string()) {
                    Entry::Occupied(mut e) => e.get_mut().update(stmt.line(), name, imm)?,
                    Entry::Vacant(e) => {
                        let _ = e.insert(Symbol::Imm(imm));
                    }
                }
            }
            Statement::AtLabel(name) => {
                map.entry(name.to_string())
                    .or_insert(Symbol::Undefined { line });
            }
            Statement::A(_) | Statement::C(_) => {}
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
