use hasm::Statement;

use crate::{Error, Module};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Executable {
    modules: HashMap<String, Module>,
}

impl Executable {
    pub fn open(module_paths: &[PathBuf]) -> Result<Self, Error> {
        let modules = module_paths
            .iter()
            .map(|path| Module::open(path).map(|module| (module.name().to_owned(), module)))
            .collect::<Result<HashMap<_, _>, _>>()?;
        if modules.is_empty() {
            return Err(Error::NoModules);
        }
        Ok(Self { modules })
    }

    pub fn translate(&self) -> Vec<Statement> {
        let mut stmts = vec![];
        for module in self.modules.values() {
            stmts.extend(module.translate());
        }
        stmts
    }
}
