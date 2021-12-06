pub use self::parser::*;
use std::{borrow::Borrow, fmt};

mod parser;

#[derive(Debug, Clone)]
pub(crate) struct Module {}

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
