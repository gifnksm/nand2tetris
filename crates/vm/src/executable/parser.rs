use super::Executable;
use crate::ParseModuleError;
use crate::{Command, FuncName, Module, ModuleName, ParseModuleErrorKind};
use std::io;
use std::{collections::BTreeMap, path::PathBuf};
use thiserror::Error;

impl Executable {
    pub fn open(module_paths: &[PathBuf]) -> Result<Self, ParseExecutableError> {
        let mut functions = FunctionTable::new();
        let modules = module_paths
            .iter()
            .map(|path| {
                Module::open(path, &mut functions).map(|module| (module.name().to_owned(), module))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        if modules.is_empty() {
            return Err(ParseExecutableError::NoModules);
        }
        let functions = functions.finish()?;
        Ok(Self { modules, functions })
    }
}

#[derive(Debug, Error)]
pub enum ParseExecutableError {
    #[error("failed to parse module: {}", _0)]
    ParseModule(ModuleName, #[source] ParseModuleError),
    #[error("invalid module path: {}", _0.display())]
    InvalidModulePath(PathBuf),
    #[error("invalid module name: {}", _0)]
    InvalidModuleName(String),
    #[error("failed to open module file: {}", _0.display())]
    ModuleOpen(PathBuf, #[source] io::Error),
    #[error("no modules found")]
    NoModules,
    #[error(
        "function is called but not defined: {} (called at {}:{})",
        _0,
        _1.path.display(),
        _1.line
    )]
    FunctionNotDefined(FuncName, FuncProp),
    #[error(
        "function artiy mismatch: {} (defined at {}:{} with arity {}, called at {}:{} with arity {}",
        _0,
        _1.path.display(),
        _1.line,
        _1.arity,
        _2.path.display(),
        _2.line,
        _2.arity
    )]
    ArityMismatch(FuncName, FuncProp, FuncProp),
    #[error("multiple function found, but there is not entry point")]
    NoEntryPoint,
}

#[derive(Debug, Clone)]
pub struct FuncProp {
    pub path: PathBuf,
    pub line: u32,
    pub arity: u8,
}

impl FuncProp {
    pub fn new(path: impl Into<PathBuf>, line: u32, arity: u8) -> Self {
        let path = path.into();
        Self { path, line, arity }
    }
}

#[derive(Debug, Clone, Default)]
struct FunctionState {
    called: Option<FuncProp>,
    defined: Option<(FuncProp, ModuleName, Vec<Command>)>,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionTable {
    functions: BTreeMap<FuncName, FunctionState>,
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
        module_name: &ModuleName,
        func_name: &FuncName,
        prop: FuncProp,
        body: Vec<Command>,
    ) -> Result<(), ParseModuleErrorKind> {
        let f = self.functions.entry(func_name.clone()).or_default();
        if let Some((defined, _, _)) = f.defined.replace((prop, module_name.clone(), body)) {
            return Err(ParseModuleErrorKind::FunctionRedefinition(
                func_name.clone(),
                defined,
            ));
        }
        Ok(())
    }

    pub(crate) fn finish(
        self,
    ) -> Result<BTreeMap<FuncName, (ModuleName, Vec<Command>)>, ParseExecutableError> {
        let mut functions = BTreeMap::new();
        self.functions
            .into_iter()
            .find_map(|(func_name, state)| match (state.defined, state.called) {
                (None, Some(called)) => Some(Err(ParseExecutableError::FunctionNotDefined(
                    func_name, called,
                ))),
                (Some((defined, _, _)), Some(called)) if defined.arity != called.arity => {
                    Some(Err(ParseExecutableError::ArityMismatch(
                        func_name, defined, called,
                    )))
                }
                (Some((_, module_name, body)), _) => {
                    functions.insert(func_name, (module_name, body));
                    None
                }
                _ => None,
            })
            .unwrap_or(Ok(()))?;

        if !functions.contains_key(&FuncName::entry_point()) && functions.len() > 1 {
            return Err(ParseExecutableError::NoEntryPoint);
        }
        Ok(functions)
    }
}
