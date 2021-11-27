use crate::{code_gen::CodeGen, Error, Ident, Module, ParseModuleErrorKind};
use hasm::Statement;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub struct Executable {
    modules: HashMap<String, Module>,
    functions: HashSet<Ident>,
}

impl Executable {
    pub fn open(module_paths: &[PathBuf]) -> Result<Self, Error> {
        let mut functions = FunctionTable::new();
        let modules = module_paths
            .iter()
            .map(|path| {
                Module::open(path, &mut functions).map(|module| (module.name().to_owned(), module))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        if modules.is_empty() {
            return Err(Error::NoModules);
        }
        let functions = functions.finish()?;
        Ok(Self { modules, functions })
    }

    pub fn translate(&self) -> Vec<Statement> {
        let mut gen = CodeGen::new("$builtin", 0);
        if let Some(entry_point) = self.functions.get("Sys.init") {
            gen.bootstrap(entry_point);
        }
        let mut stmts = gen.into_statements();
        for module in self.modules.values() {
            stmts.extend(module.translate());
        }
        stmts
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Function {
    called: Option<(String, u32)>,
    defined: Option<(String, u32)>,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionTable {
    functions: HashMap<Ident, Function>,
}

impl FunctionTable {
    pub(crate) fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    pub(crate) fn call(
        &mut self,
        name: &Ident,
        module_name: &str,
        line: u32,
    ) -> Result<(), ParseModuleErrorKind> {
        let f = self.functions.entry(name.clone()).or_default();
        if f.called.is_none() {
            f.called = Some((module_name.to_owned(), line));
        }
        Ok(())
    }

    pub(crate) fn define(
        &mut self,
        name: &Ident,
        module_name: &str,
        line: u32,
    ) -> Result<(), ParseModuleErrorKind> {
        let f = self.functions.entry(name.clone()).or_default();
        if let Some(defined) = f.defined.replace((module_name.into(), line)) {
            return Err(ParseModuleErrorKind::FunctionRedefinition(
                name.as_str().to_owned(),
                defined.0.clone(),
                defined.1,
            ));
        }
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<HashSet<Ident>, Error> {
        let mut functions = HashSet::new();
        self.functions
            .into_iter()
            .find_map(|(name, state)| match (state.defined, state.called) {
                (None, Some(called)) => Some(Err(Error::FunctionNotDefined(
                    name.as_str().to_owned(),
                    called.0,
                    called.1,
                ))),
                _ => {
                    functions.insert(name);
                    None
                }
            })
            .unwrap_or(Ok(()))?;
        Ok(functions)
    }
}
