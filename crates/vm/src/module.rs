pub use self::parser::*;
use crate::{Command, FunctionTable, ParseExecutableError};
use std::{
    borrow::Borrow,
    fmt,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    str::FromStr,
};

mod parser;

#[derive(Debug, Clone)]
pub(crate) struct Module {
    name: ModuleName,
    path: PathBuf,
    commands: Vec<Command>,
}

impl Module {
    pub(crate) fn open(
        path: &Path,
        functions: &mut FunctionTable,
    ) -> Result<Self, ParseExecutableError> {
        let path = path.to_owned();
        let name = path
            .with_extension("")
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ParseExecutableError::InvalidModulePath(path.clone()))
            .and_then(ModuleName::from_str)?;

        let file =
            File::open(&path).map_err(|e| ParseExecutableError::ModuleOpen(path.clone(), e))?;
        let reader = BufReader::new(file);
        Self::from_reader(name, path, reader, functions)
    }

    pub(crate) fn name(&self) -> &ModuleName {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleName(String);

impl Borrow<str> for ModuleName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModuleName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl ModuleName {
    pub(crate) fn builtin() -> Self {
        Self("$builtin".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
