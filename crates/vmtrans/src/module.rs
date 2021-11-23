use crate::{Command, Error, ParseCommandError};
use hasm::Statement;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    str::FromStr,
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub(crate) struct Module {
    name: String,
    commands: Vec<Command>,
}

impl Module {
    pub(crate) fn open(path: &Path) -> Result<Self, Error> {
        let name = path
            .with_extension("")
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| Error::InvalidModulePath(path.into()))?;
        if !Self::is_valid_name(&name) {
            return Err(Error::InvalidModuleName(name));
        }

        let file = File::open(path).map_err(|e| Error::ModuleOpen(path.into(), e))?;
        let reader = BufReader::new(file);
        Self::from_reader(name, reader)
    }

    pub(crate) fn from_reader(name: String, mut reader: impl BufRead) -> Result<Self, Error> {
        if !Self::is_valid_name(&name) {
            return Err(Error::InvalidModuleName(name));
        }

        let mut commands = vec![];
        let mut line_buf = String::new();
        for line in 1.. {
            line_buf.clear();
            let res = reader
                .read_line(&mut line_buf)
                .map_err(|e| Error::ParseModule(name.clone(), ParseModuleError::new(line, e)))?;
            if res == 0 {
                break;
            }

            if let Some(command) = parse_line(&line_buf)
                .map_err(|e| Error::ParseModule(name.clone(), ParseModuleError::new(line, e)))?
            {
                commands.push(command);
            }
        }
        Ok(Self { name, commands })
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn translate(&self) -> Vec<Statement> {
        let mut stmts = vec![];
        for (index, command) in self.commands.iter().enumerate() {
            stmts.extend(command.translate(&self.name, index));
        }
        stmts
    }

    fn is_valid_name(name: &str) -> bool {
        let mut cs = name.chars();
        cs.next()
            .map(|ch| ch.is_ascii_alphabetic())
            .unwrap_or(false)
            && cs.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}

#[derive(Debug, Error)]
#[error("syntax error at line {}", line)]
pub struct ParseModuleError {
    line: u32,
    #[source]
    kind: ParseModuleErrorKind,
}

impl ParseModuleError {
    pub(crate) fn new(line: u32, kind: impl Into<ParseModuleErrorKind>) -> Self {
        let kind = kind.into();
        Self { line, kind }
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
}
