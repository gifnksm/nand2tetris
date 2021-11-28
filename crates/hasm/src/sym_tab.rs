use crate::{Error, ErrorKind, Imm, Label, Statement, StatementWithLine};
use indexmap::{map::Entry, IndexMap};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum Symbol {
    Defined(u32),
    Undefined { line: u32 },
}

impl Symbol {
    fn update(&mut self, line: u32, name: &Label, value: u32) -> Result<(), Error> {
        match self {
            Symbol::Undefined { .. } => *self = Symbol::Defined(value),
            Symbol::Defined(_) => {
                return Err(Error::new(line, ErrorKind::DuplicateLabel(name.clone())))
            }
        }
        Ok(())
    }
}

pub(crate) fn from_stmts(stmts: &[StatementWithLine]) -> Result<HashMap<String, u16>, Error> {
    let mut symbol_map = predefined_map();
    let mut last_inst_count = insert_symbols(&mut symbol_map, stmts)?;
    let mut map = create_map(symbol_map)?;

    loop {
        let (updated, inst_count) = update_map(&mut map, stmts);
        if !updated && inst_count == last_inst_count {
            break;
        }
        assert!(!updated || inst_count < last_inst_count);
        last_inst_count = inst_count;
    }

    if last_inst_count > u32::from(u16::MAX) {
        return Err(Error::new(0, ErrorKind::TooLargeProgram));
    }
    map.into_iter()
        .map(|(k, v)| {
            if let Ok(v) = u16::try_from(v) {
                Ok((k, v))
            } else {
                Err(Error::new(0, ErrorKind::TooLargeProgram))
            }
        })
        .collect()
}

fn predefined_map() -> IndexMap<String, Symbol> {
    let predefined = [
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
        map.insert(name.to_string(), Symbol::Defined(u32::from(value.value())));
    }
    map
}

fn inst_size_for_a(value: u32) -> u32 {
    if value > u32::from(Imm::MAX.value()) {
        2
    } else {
        1
    }
}

fn insert_symbols(
    map: &mut IndexMap<String, Symbol>,
    stmts: &[StatementWithLine],
) -> Result<u32, Error> {
    let mut inst_count = 0;
    for stmt in stmts {
        let line = stmt.line();
        let kind = stmt.data();
        let inst_size = match kind {
            Statement::Label(_) => 0,
            Statement::AtLabel(_) => 2, // assume all @LABEL translates to 2 instructions
            Statement::A(n) => inst_size_for_a(u32::from(*n)),
            Statement::C(_) => 1,
        };
        inst_count += inst_size;

        match kind {
            Statement::Label(name) => match map.entry(name.to_string()) {
                Entry::Occupied(mut e) => e.get_mut().update(stmt.line(), name, inst_count)?,
                Entry::Vacant(e) => {
                    let _ = e.insert(Symbol::Defined(inst_count));
                }
            },
            Statement::AtLabel(name) => {
                map.entry(name.to_string())
                    .or_insert(Symbol::Undefined { line });
            }
            Statement::A(_) | Statement::C(_) => {}
        }
    }

    Ok(inst_count)
}

fn create_map(map: IndexMap<String, Symbol>) -> Result<HashMap<String, u32>, Error> {
    let mut result = HashMap::new();
    let mut value = 0x0010;
    for (name, symbol) in map {
        match symbol {
            Symbol::Defined(value) => {
                result.insert(name, value);
            }
            Symbol::Undefined { line } => {
                if value > 255 {
                    return Err(Error::new(line, ErrorKind::TooManySymbols(name)));
                }
                result.insert(name, value);
                value += 1;
            }
        }
    }
    Ok(result)
}

fn update_map(map: &mut HashMap<String, u32>, stmts: &[StatementWithLine]) -> (bool, u32) {
    let mut updated = false;
    let mut inst_count = 0;
    for stmt in stmts {
        let kind = stmt.data();
        let inst_size = match kind {
            Statement::Label(_) => 0,
            Statement::AtLabel(label) => inst_size_for_a(*map.get(label.as_str()).unwrap()),
            Statement::A(n) => inst_size_for_a(u32::from(*n)),
            Statement::C(_) => 1,
        };
        inst_count += inst_size;

        // update by new value
        if let Statement::Label(name) = kind {
            let old = map.insert(name.to_string(), inst_count).unwrap();
            if old != inst_count {
                updated = true;
            }
        }
    }
    (updated, inst_count)
}
