use crate::{code_gen::CodeGen, Error, FuncProp, Ident, Module, ParseModuleErrorKind};
use hasm::{Imm, Statement};
use std::{
    collections::{BTreeMap, HashSet},
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub struct Executable {
    modules: BTreeMap<String, Module>,
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
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        if modules.is_empty() {
            return Err(Error::NoModules);
        }
        let functions = functions.finish()?;
        Ok(Self { modules, functions })
    }

    pub fn translate(&self) -> Vec<Statement> {
        let mut gen = CodeGen::new("$builtin", "bootstrap", 0);
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
    called: Option<FuncProp>,
    defined: Option<FuncProp>,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionTable {
    functions: BTreeMap<Ident, Function>,
}

impl FunctionTable {
    pub(crate) fn new() -> Self {
        Self {
            functions: BTreeMap::new(),
        }
    }

    pub(crate) fn call(
        &mut self,
        name: &Ident,
        prop: FuncProp,
    ) -> Result<(), ParseModuleErrorKind> {
        let f = self.functions.entry(name.clone()).or_default();
        let arity = prop.arity;
        let called = f.called.get_or_insert(prop);
        if called.arity != arity {
            return Err(ParseModuleErrorKind::CallerArityMismatch(
                name.as_str().to_owned(),
                arity,
                called.clone(),
            ));
        }
        Ok(())
    }

    pub(crate) fn define(
        &mut self,
        name: &Ident,
        prop: FuncProp,
    ) -> Result<(), ParseModuleErrorKind> {
        let f = self.functions.entry(name.clone()).or_default();
        if let Some(defined) = f.defined.replace(prop) {
            return Err(ParseModuleErrorKind::FunctionRedefinition(
                name.as_str().to_owned(),
                defined,
            ));
        }
        Ok(())
    }

    pub(crate) fn arg_access(&mut self, name: &Ident, index: &Imm) {
        let prop = self
            .functions
            .get_mut(name)
            .unwrap()
            .defined
            .as_mut()
            .unwrap();
        prop.arity = u8::max(prop.arity, u8::try_from(index.value()).unwrap() + 1);
    }

    pub(crate) fn finish(self) -> Result<HashSet<Ident>, Error> {
        let mut functions = HashSet::new();
        self.functions
            .into_iter()
            .find_map(|(name, state)| match (state.defined, state.called) {
                (None, Some(called)) => Some(Err(Error::FunctionNotDefined(
                    name.as_str().to_owned(),
                    called,
                ))),
                (Some(defined), Some(called)) if defined.arity != called.arity => Some(Err(
                    Error::ArityMismatch(name.as_str().to_owned(), defined, called),
                )),
                _ => {
                    functions.insert(name);
                    None
                }
            })
            .unwrap_or(Ok(()))?;
        Ok(functions)
    }
}
