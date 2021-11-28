use crate::{
    code_gen::CodeGen, Error, FuncName, FuncProp, Module, ModuleName, ParseModuleErrorKind,
};
use hasm::Statement;
use std::{
    collections::{BTreeMap, HashSet},
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub struct Executable {
    modules: BTreeMap<ModuleName, Module>,
    functions: HashSet<FuncName>,
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
        let mut stmts = self.bootstrap();
        for module in self.modules.values() {
            stmts.extend(module.translate());
        }
        stmts
    }

    fn bootstrap(&self) -> Vec<Statement> {
        let module_name = ModuleName::builtin();
        let func_name = FuncName::bootstrap();
        let mut gen = CodeGen::new(&module_name, &func_name, 0);
        if let Some(entry_point) = self.functions.get("Sys.init") {
            gen.bootstrap(entry_point);
        }
        gen.into_statements()
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Function {
    called: Option<FuncProp>,
    defined: Option<FuncProp>,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionTable {
    functions: BTreeMap<FuncName, Function>,
}

impl FunctionTable {
    pub(crate) fn new() -> Self {
        Self {
            functions: BTreeMap::new(),
        }
    }

    pub(crate) fn call(
        &mut self,
        name: &FuncName,
        prop: FuncProp,
    ) -> Result<(), ParseModuleErrorKind> {
        let f = self.functions.entry(name.clone()).or_default();
        let arity = prop.arity;
        let called = f.called.get_or_insert(prop);
        if called.arity != arity {
            return Err(ParseModuleErrorKind::CallerArityMismatch(
                name.clone(),
                arity,
                called.clone(),
            ));
        }
        Ok(())
    }

    pub(crate) fn define(
        &mut self,
        name: &FuncName,
        prop: FuncProp,
    ) -> Result<(), ParseModuleErrorKind> {
        let f = self.functions.entry(name.clone()).or_default();
        if let Some(defined) = f.defined.replace(prop) {
            return Err(ParseModuleErrorKind::FunctionRedefinition(
                name.clone(),
                defined,
            ));
        }
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<HashSet<FuncName>, Error> {
        let mut functions = HashSet::new();
        self.functions
            .into_iter()
            .find_map(|(name, state)| match (state.defined, state.called) {
                (None, Some(called)) => Some(Err(Error::FunctionNotDefined(name, called))),
                (Some(defined), Some(called)) if defined.arity != called.arity => {
                    Some(Err(Error::ArityMismatch(name, defined, called)))
                }
                _ => {
                    functions.insert(name);
                    None
                }
            })
            .unwrap_or(Ok(()))?;
        Ok(functions)
    }
}
