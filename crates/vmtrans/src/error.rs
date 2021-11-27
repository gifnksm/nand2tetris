use crate::ParseModuleError;
use std::{io, path::PathBuf};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse module: {}", _0)]
    ParseModule(String, #[source] ParseModuleError),
    #[error("invalid module path: {}", _0.display())]
    InvalidModulePath(PathBuf),
    #[error("invalid module name: {}", _0)]
    InvalidModuleName(String),
    #[error("failed to open module file: {}", _0.display())]
    ModuleOpen(PathBuf, #[source] io::Error),
    #[error("no modules found")]
    NoModules,
    #[error(
        "function is called but not defined: {} (first called at module {} line {})",
        _0,
        _1,
        _2
    )]
    FunctionNotDefined(String, String, u32),
}
