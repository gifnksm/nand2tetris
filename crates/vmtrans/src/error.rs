use crate::{FuncName, ModuleName, ParseModuleError};
use std::{io, path::PathBuf};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
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
