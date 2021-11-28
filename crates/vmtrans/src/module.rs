use crate::{Command, Error, FuncName, FuncProp, FunctionTable, Label, ParseCommandError, Segment};
use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    str::FromStr,
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub(crate) struct Module {
    name: ModuleName,
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
            .ok_or_else(|| Error::InvalidModulePath(path.clone()))
            .and_then(ModuleName::from_str)?;

        let file = File::open(&path).map_err(|e| Error::ModuleOpen(path.clone(), e))?;
        let reader = BufReader::new(file);
        Self::from_reader(name, path, reader, functions)
    }

    pub(crate) fn from_reader(
        name: ModuleName,
        path: PathBuf,
        mut reader: impl BufRead,
        functions: &mut FunctionTable,
    ) -> Result<Self, Error> {
        let mut parser = Parser::new(&path, &name, functions);
        let mut line_buf = String::new();
        for line in 1.. {
            line_buf.clear();
            let res = reader.read_line(&mut line_buf).map_err(|e| {
                Error::ParseModule(name.clone(), ParseModuleError::new(path.clone(), line, e))
            })?;
            if res == 0 {
                let commands = parser.finish(line).map_err(|e| {
                    Error::ParseModule(name.clone(), ParseModuleError::new(path.clone(), line, e))
                })?;
                return Ok(Self {
                    name,
                    path,
                    commands,
                });
            }
            parser.parse_line(&line_buf, line).map_err(|e| {
                Error::ParseModule(name.clone(), ParseModuleError::new(path.clone(), line, e))
            })?;
        }
        unreachable!()
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

impl FromStr for ModuleName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cs = s.chars();
        let is_valid = cs
            .next()
            .map(|ch| ch.is_ascii_alphabetic())
            .unwrap_or(false)
            && cs.all(|ch| ch.is_ascii_alphanumeric() || ch == '_');
        if !is_valid {
            return Err(Error::InvalidModuleName(s.to_owned()));
        }
        Ok(Self(s.to_owned()))
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

#[derive(Debug)]
struct Parser<'a> {
    path: &'a Path,
    module_name: &'a ModuleName,
    functions: &'a mut FunctionTable,
    func_name: FuncName,
    num_locals: u8,
    arity: u8,
    labels: LabelTable,
    commands: Vec<Command>,
}

impl<'a> Parser<'a> {
    fn new(path: &'a Path, module_name: &'a ModuleName, functions: &'a mut FunctionTable) -> Self {
        Self {
            path,
            module_name,
            functions,
            func_name: FuncName::toplevel(),
            num_locals: u8::MAX, // workaround: consider toplevel functions as having 256 local variables
            arity: 0,
            labels: LabelTable::new(),
            commands: vec![],
        }
    }

    fn parse_line(&mut self, s: &str, line: u32) -> Result<(), ParseModuleErrorKind> {
        let command = if let Some(command) = parse_line(s)? {
            command
        } else {
            return Ok(());
        };

        match &command {
            Command::Label(label) => self.labels.define(label, line)?,
            Command::Goto(label) | Command::IfGoto(label) => self.labels.use_(label, line),
            Command::Function(func_name, num_locals) => {
                self.finish_func(line)?;
                self.start_func(func_name.clone(), *num_locals);
            }
            Command::Call(func_name, arity) => {
                self.functions
                    .call(func_name, FuncProp::new(&self.path, line, *arity))?;
            }
            Command::Push(Segment::Local, index) | Command::Pop(Segment::Local, index)
                if *index >= u16::from(self.num_locals) =>
            {
                return Err(
                    ParseCommandError::TooLargeIndex(*index, u16::from(self.num_locals)).into(),
                )
            }
            Command::Push(Segment::Argument, index) | Command::Pop(Segment::Argument, index) => {
                self.arity = u8::max(self.arity, u8::try_from(index + 1).unwrap());
            }
            _ => {}
        }
        self.commands.push(command);

        Ok(())
    }

    fn start_func(&mut self, func_name: FuncName, num_locals: u8) {
        self.func_name = func_name;
        self.num_locals = num_locals;
        self.arity = 0;
    }

    fn finish_func(&mut self, line: u32) -> Result<(), ParseModuleErrorKind> {
        self.labels.finish()?;
        if let Some(last) = self.commands.last() {
            if !self.func_name.is_toplevel() && !last.is_jump() {
                return Err(ParseModuleErrorKind::NoJumpCommandAtEndOfFunction(
                    self.func_name.clone(),
                ));
            }
            let prop = FuncProp::new(&self.path, line, self.arity);
            let body = self.commands.drain(..).collect();
            self.functions
                .define(self.module_name, &self.func_name, prop, body)?;
        }
        Ok(())
    }

    fn finish(mut self, line: u32) -> Result<Vec<Command>, ParseModuleErrorKind> {
        self.finish_func(line)?;
        Ok(self.commands)
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
    LabelRedefinition(Label, u32),
    #[error("label is used but not defined: {} (first used at line {})", _0, _1)]
    LabelNotDefined(Label, u32),
    #[error(
        "function redefined: {} (first defined at {}:{})",
        _0,
        _1.path.display(),
        _1.line
    )]
    FunctionRedefinition(FuncName, FuncProp),
    #[error(
        "function called with different arity: {} with arity {} (first called at {}:{} with arity {})",
        _0,
        _1,
        _2.path.display(),
        _2.line,
        _2.arity
    )]
    CallerArityMismatch(FuncName, u8, FuncProp),
    #[error("no jump command at end of function: {}", _0)]
    NoJumpCommandAtEndOfFunction(FuncName),
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
    labels: BTreeMap<Label, LabelState>,
}

impl LabelTable {
    fn new() -> Self {
        Self {
            labels: BTreeMap::new(),
        }
    }

    fn define(&mut self, label: &Label, line: u32) -> Result<(), ParseModuleErrorKind> {
        let s = self.labels.entry(label.clone()).or_default();
        if let Some(defined) = s.defined.replace(line) {
            return Err(ParseModuleErrorKind::LabelRedefinition(
                label.clone(),
                defined,
            ));
        }
        Ok(())
    }

    fn use_(&mut self, label: &Label, line: u32) {
        let s = self.labels.entry(label.clone()).or_default();
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
        assert!(ModuleName::from_str("foo").is_ok());
        assert!(ModuleName::from_str("foo_bar").is_ok());
        assert!(ModuleName::from_str("foo_bar_baz").is_ok());
        assert!(ModuleName::from_str("foo123").is_ok());
        assert!(ModuleName::from_str("").is_err());
        assert!(ModuleName::from_str("foo bar").is_err());
        assert!(ModuleName::from_str("123foo").is_err());
    }

    #[test]
    fn parse_label() {
        fn p(input: &[&str]) -> Result<Module, Error> {
            Module::from_reader(
                ModuleName::from_str("foo").unwrap(),
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
            ) if l.as_str() == "foo"
        ));
        assert!(matches!(
            p(&["label foo", "goto foo", "goto bar"]).unwrap_err(),
            Error::ParseModule(
                _,
                ParseModuleError {
                    kind: ParseModuleErrorKind::LabelNotDefined(l, _),
                    ..
                }
            ) if l.as_str() == "bar"
        ));
    }
}
