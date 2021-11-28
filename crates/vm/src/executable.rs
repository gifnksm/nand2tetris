pub(crate) use self::parser::*;
use crate::{Command, FuncName, Module, ModuleName};
use std::collections::BTreeMap;

mod parser;
mod translator;

#[derive(Debug, Clone)]
pub struct Executable {
    modules: BTreeMap<ModuleName, Module>,
    functions: BTreeMap<FuncName, (ModuleName, Vec<Command>)>,
}
