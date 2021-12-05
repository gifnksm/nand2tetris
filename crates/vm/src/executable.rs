pub(crate) use self::parser::*;
use crate::{Command, FuncName, ModuleName};
use std::collections::BTreeMap;

mod parser;
mod translator;

#[derive(Debug, Clone)]
pub struct Executable {
    functions: BTreeMap<FuncName, (ModuleName, Vec<Command>)>,
}
