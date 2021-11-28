use crate::{Command, Error, FuncProp, FunctionTable, Ident, ParseCommandError, Segment};
use hasm::Statement;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    str::FromStr,
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub(crate) struct Module {
    name: String,
    path: PathBuf,
    commands: Vec<Command>,
}

impl Module {
    pub(crate) fn open(path: &Path, functions: &mut FunctionTable) -> Result<Self, Error> {
        let path = path.to_owned();
        let name = path
            .with_extension("")
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| Error::InvalidModulePath(path.clone()))?;
        if !Self::is_valid_name(&name) {
            return Err(Error::InvalidModuleName(name));
        }

        let file = File::open(&path).map_err(|e| Error::ModuleOpen(path.clone(), e))?;
        let reader = BufReader::new(file);
        Self::from_reader(name, path, reader, functions)
    }

    pub(crate) fn from_reader(
        name: String,
        path: PathBuf,
        mut reader: impl BufRead,
        functions: &mut FunctionTable,
    ) -> Result<Self, Error> {
        if !Self::is_valid_name(&name) {
            return Err(Error::InvalidModuleName(name));
        }

        let mut func_name = None;
        let mut num_locals = u8::MAX; // workaround: consider toplevel functions as having 256 local variables
        let mut labels = LabelTable::new();
        let mut commands = vec![];
        let mut line_buf = String::new();
        for line in 1.. {
            line_buf.clear();
            let res = reader.read_line(&mut line_buf).map_err(|e| {
                Error::ParseModule(name.clone(), ParseModuleError::new(path.clone(), line, e))
            })?;
            if res == 0 {
                labels.finish().map_err(|e| {
                    Error::ParseModule(name.clone(), ParseModuleError::new(path.clone(), line, e))
                })?;
                break;
            }

            if let Some(command) = parse_line(&line_buf).map_err(|e| {
                Error::ParseModule(name.clone(), ParseModuleError::new(path.clone(), line, e))
            })? {
                match &command {
                    Command::Label(label) => labels.define(label, line),
                    Command::Goto(label) | Command::IfGoto(label) => {
                        labels.use_(label, line);
                        Ok(())
                    }
                    Command::Function(function_name, n) => labels.finish().and_then(|()| {
                        func_name = Some(function_name.clone());
                        num_locals = *n;
                        functions.define(function_name, FuncProp::new(&path, line, 0))
                    }),
                    Command::Call(function_name, arity) => {
                        functions.call(function_name, FuncProp::new(&path, line, *arity))
                    }
                    Command::Push(Segment::Local, index) | Command::Pop(Segment::Local, index)
                        if index.value() >= u16::from(num_locals) =>
                    {
                        Err(
                            ParseCommandError::TooLargeIndex(index.value(), u16::from(num_locals))
                                .into(),
                        )
                    }
                    Command::Push(Segment::Argument, index)
                    | Command::Pop(Segment::Argument, index) => {
                        if let Some(func_name) = &func_name {
                            functions.arg_access(func_name, index);
                        }
                        Ok(())
                    }
                    _ => Ok(()),
                }
                .map_err(|e| {
                    Error::ParseModule(name.clone(), ParseModuleError::new(path.clone(), line, e))
                })?;

                commands.push(command);
            }
        }

        Ok(Self {
            name,
            path,
            commands,
        })
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn translate(&self) -> Vec<Statement> {
        let mut func_name = "$toplevel";
        let mut stmts = vec![];
        for (index, command) in self.commands.iter().enumerate() {
            if let Command::Function(name, _) = command {
                func_name = name.as_str();
            }
            stmts.extend(command.translate(&self.name, func_name, index));
        }
        stmts
    }

    fn is_valid_name(name: &str) -> bool {
        let mut cs = name.chars();
        cs.next()
            .map(|ch| ch.is_ascii_alphabetic())
            .unwrap_or(false)
            && cs.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    }
}

#[derive(Debug, Error)]
#[error("syntax error at {}:{}", path.display(), line)]
pub struct ParseModuleError {
    path: PathBuf,
    line: u32,
    #[source]
    kind: ParseModuleErrorKind,
}

impl ParseModuleError {
    pub(crate) fn new(path: PathBuf, line: u32, kind: impl Into<ParseModuleErrorKind>) -> Self {
        let kind = kind.into();
        Self { path, line, kind }
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn kind(&self) -> &ParseModuleErrorKind {
        &self.kind
    }
}

#[derive(Debug, Error)]
pub enum ParseModuleErrorKind {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error(transparent)]
    ParseCommand(#[from] ParseCommandError),
    #[error("label redefined: {} (first defined at line {})", _0, _1)]
    LabelRedefinition(String, u32),
    #[error("label is used but not defined: {} (first used at line {})", _0, _1)]
    LabelNotDefined(String, u32),
    #[error(
        "function redefined: {} (first defined at {}:{})",
        _0,
        _1.path.display(),
        _1.line
    )]
    FunctionRedefinition(String, FuncProp),
    #[error(
        "function called with different arity: {} with arity {} (first called at {}:{} with arity {})",
        _0,
        _1,
        _2.path.display(),
        _2.line,
        _2.arity
    )]
    CallerArityMismatch(String, u8, FuncProp),
}

fn parse_line(s: &str) -> Result<Option<Command>, ParseModuleErrorKind> {
    let s = trim_spaces_or_comment(s);
    if s.is_empty() {
        return Ok(None);
    }

    Ok(Some(Command::from_str(s)?))
}

fn trim_spaces_or_comment(s: &str) -> &str {
    if let Some((pre, _post)) = s.split_once("//") {
        pre.trim()
    } else {
        s.trim()
    }
}

#[derive(Debug, Clone, Default)]
struct LabelState {
    used: Option<u32>,
    defined: Option<u32>,
}

#[derive(Debug)]
struct LabelTable {
    labels: BTreeMap<String, LabelState>,
}

impl LabelTable {
    fn new() -> Self {
        Self {
            labels: BTreeMap::new(),
        }
    }

    fn define(&mut self, label: &Ident, line: u32) -> Result<(), ParseModuleErrorKind> {
        let s = self.labels.entry(label.as_str().to_string()).or_default();
        if let Some(defined) = s.defined.replace(line) {
            return Err(ParseModuleErrorKind::LabelRedefinition(
                label.as_str().to_string(),
                defined,
            ));
        }
        Ok(())
    }

    fn use_(&mut self, label: &Ident, line: u32) {
        let s = self.labels.entry(label.as_str().to_string()).or_default();
        if s.used.is_none() {
            s.used = Some(line);
        }
    }

    fn finish(&mut self) -> Result<(), ParseModuleErrorKind> {
        for (label, state) in &self.labels {
            if let (None, Some(used_line)) = (state.defined, state.used) {
                return Err(ParseModuleErrorKind::LabelNotDefined(
                    label.clone(),
                    used_line,
                ));
            }
        }
        self.labels.clear();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_valid_name() {
        assert!(Module::is_valid_name("foo"));
        assert!(Module::is_valid_name("foo_bar"));
        assert!(Module::is_valid_name("foo_bar_baz"));
        assert!(Module::is_valid_name("foo123"));
        assert!(!Module::is_valid_name(""));
        assert!(!Module::is_valid_name("foo bar"));
        assert!(!Module::is_valid_name("123foo"));
    }

    #[test]
    fn parse_label() {
        fn p(input: &[&str]) -> Result<Module, Error> {
            Module::from_reader(
                "foo".into(),
                "foo.vm".into(),
                BufReader::new(input.join("\n").into_bytes().as_slice()),
                &mut FunctionTable::new(),
            )
        }

        assert!(p(&["label foo", "label bar", "goto foo", "if-goto bar"]).is_ok());
        assert!(matches!(
            p(&["label foo", "label bar", "label foo"]).unwrap_err(),
            Error::ParseModule(
                _,
                ParseModuleError {
                    kind: ParseModuleErrorKind::LabelRedefinition(l, _),
                    ..
                }
            ) if l == "foo"
        ));
        assert!(matches!(
            p(&["label foo", "goto foo", "goto bar"]).unwrap_err(),
            Error::ParseModule(
                _,
                ParseModuleError {
                    kind: ParseModuleErrorKind::LabelNotDefined(l, _),
                    ..
                }
            ) if l == "bar"
        ));
    }
}
