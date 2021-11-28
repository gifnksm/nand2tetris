use super::Executable;
use crate::{Label, Statement};
use hack::{Imm, Instruction};
use indexmap::IndexMap;
use std::collections::HashMap;
use thiserror::Error;

impl Executable {
    pub fn assemble(&self) -> Result<Vec<Instruction>, AssembleExecutableError> {
        let mut symbols = predefined_symbols();
        let mut last_inst_count = self.insert_symbols(&mut symbols);
        let mut symbols = assign_undefined_symbols(symbols)?;
        loop {
            let (updated, inst_count) = self.update_symbols(&mut symbols);
            if !updated && inst_count == last_inst_count {
                break;
            }
            assert!(!updated || inst_count < last_inst_count);
            last_inst_count = inst_count;
        }

        if last_inst_count > u32::from(u16::MAX) {
            return Err(AssembleExecutableError::TooLargeProgram);
        }
        let symbols = symbols
            .into_iter()
            .map(|(k, v)| (k, u16::try_from(v).unwrap()))
            .collect();

        let mut insts = vec![];
        for stmt in &self.stmts {
            stmt.assemble(&symbols, &mut insts);
        }

        Ok(insts)
    }

    fn insert_symbols(&self, map: &mut IndexMap<Label, Symbol>) -> u32 {
        let mut inst_count = 0;
        for stmt in &self.stmts {
            let inst_size = match stmt {
                Statement::Label(_) => 0,
                Statement::AtLabel(_) => 2, // assume all @LABEL translates to 2 instructions
                Statement::A(n) => inst_size_for_a(u32::from(*n)),
                Statement::C(_) => 1,
            };
            inst_count += inst_size;

            match stmt {
                Statement::Label(name) => {
                    let old = map.insert(name.clone(), Symbol::Defined(inst_count));
                    assert!(matches!(old, None | Some(Symbol::Undefined)));
                }
                Statement::AtLabel(name) => {
                    map.entry(name.clone()).or_insert(Symbol::Undefined);
                }
                Statement::A(_) | Statement::C(_) => {}
            }
        }
        inst_count
    }

    fn update_symbols(&self, symbols: &mut HashMap<Label, u32>) -> (bool, u32) {
        let mut updated = false;
        let mut inst_count = 0;
        for stmt in &self.stmts {
            let inst_size = match stmt {
                Statement::Label(_) => 0,
                Statement::AtLabel(label) => inst_size_for_a(*symbols.get(label).unwrap()),
                Statement::A(n) => inst_size_for_a(u32::from(*n)),
                Statement::C(_) => 1,
            };
            inst_count += inst_size;

            if let Statement::Label(name) = stmt {
                // update by new value
                let old = symbols.insert(name.clone(), inst_count).unwrap();
                if old != inst_count {
                    updated = true;
                }
            }
        }
        (updated, inst_count)
    }
}

#[derive(Debug, Error)]
pub enum AssembleExecutableError {
    #[error("too large program")]
    TooLargeProgram,
    #[error("too many symbols: {}", _0)]
    TooManySymbols(Label),
}

fn predefined_symbols() -> IndexMap<Label, Symbol> {
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
        map.insert(name, Symbol::Defined(u32::from(value.value())));
    }
    map
}

#[derive(Debug, Clone)]
enum Symbol {
    Defined(u32),
    Undefined,
}

fn inst_size_for_a(value: u32) -> u32 {
    if value > u32::from(Imm::MAX.value()) {
        2
    } else {
        1
    }
}

fn assign_undefined_symbols(
    symbols: IndexMap<Label, Symbol>,
) -> Result<HashMap<Label, u32>, AssembleExecutableError> {
    let mut result = HashMap::new();
    let mut next_value = 0x0010;
    for (name, symbol) in symbols {
        match symbol {
            Symbol::Defined(value) => {
                result.insert(name, value);
            }
            Symbol::Undefined => {
                if next_value > 255 {
                    return Err(AssembleExecutableError::TooManySymbols(name));
                }
                result.insert(name, next_value);
                next_value += 1;
            }
        }
    }
    Ok(result)
}
