use super::Executable;
use crate::{Label, ParseStatementError, Statement};
use std::{
    collections::{hash_map::Entry, HashMap},
    io::{self, prelude::*},
    str::FromStr,
};
use thiserror::Error;

impl Executable {
    pub fn from_reader(mut reader: impl BufRead) -> Result<Self, ReadExecutableError> {
        let mut stmts = vec![];
        let mut symbols: HashMap<Label, Symbol> = HashMap::new();

        let mut line_buf = String::new();
        for line in 1.. {
            line_buf.clear();
            let res = reader
                .read_line(&mut line_buf)
                .map_err(|e| ReadExecutableError::new(line, e))?;
            if res == 0 {
                break;
            }

            if let Some(stmt) =
                parse_line(&line_buf).map_err(|e| ReadExecutableError::new(line, e))?
            {
                match &stmt {
                    Statement::Label(name) => match symbols.entry(name.clone()) {
                        Entry::Occupied(mut e) => e.get_mut().update(line, name)?,
                        Entry::Vacant(e) => {
                            let _ = e.insert(Symbol::Defined { line });
                        }
                    },
                    Statement::AtLabel(name) => {
                        symbols
                            .entry(name.clone())
                            .or_insert(Symbol::Undefined { line });
                    }
                    Statement::A(_) | Statement::C(_) => {}
                }
                stmts.push(stmt);
            }
        }

        Ok(Executable { stmts })
    }
}

fn parse_line(line: &str) -> Result<Option<Statement>, ReadExecutableErrorKind> {
    let line = trim_spaces_or_comment(line);
    if line.is_empty() {
        return Ok(None);
    }

    Ok(Some(Statement::from_str(line)?))
}

fn trim_spaces_or_comment(s: &str) -> &str {
    if let Some((pre, _post)) = s.split_once("//") {
        pre.trim()
    } else {
        s.trim()
    }
}

#[derive(Debug, Clone)]
enum Symbol {
    Defined { line: u32 },
    Undefined { line: u32 },
}

impl Symbol {
    fn update(&mut self, line: u32, name: &Label) -> Result<(), ReadExecutableError> {
        match self {
            Symbol::Undefined { .. } => *self = Symbol::Defined { line },
            Symbol::Defined { line: prev_line } => {
                return Err(ReadExecutableError::new(
                    line,
                    ReadExecutableErrorKind::DuplicateLabel(name.clone(), *prev_line),
                ))
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
#[error("syntax error at line {}", line)]
pub struct ReadExecutableError {
    line: u32,
    #[source]
    kind: ReadExecutableErrorKind,
}

impl ReadExecutableError {
    fn new(line: u32, kind: impl Into<ReadExecutableErrorKind>) -> Self {
        let kind = kind.into();
        Self { line, kind }
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn kind(&self) -> &ReadExecutableErrorKind {
        &self.kind
    }
}

#[derive(Debug, Error)]
pub enum ReadExecutableErrorKind {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error(transparent)]
    InvalidStatement(#[from] ParseStatementError),
    #[error("duplicated label: {} (first defined at line {})", _0, _1)]
    DuplicateLabel(Label, u32),
}

#[cfg(test)]
mod tests {
    #[test]
    fn trim_spaces_or_comment() {
        use super::trim_spaces_or_comment as t;
        assert_eq!(t(""), "");
        assert_eq!(t(" aaa \n"), "aaa");
        assert_eq!(t("foo bar // baz"), "foo bar");
        assert_eq!(t("foo bar baz //\n"), "foo bar baz");
    }

    #[test]
    fn parse_line() {
        use super::parse_line as p;
        assert_eq!(p("").unwrap(), None);
        assert_eq!(p("// foo").unwrap(), None);
        assert_eq!(
            p("M=M+1;JEQ // comment").unwrap().unwrap().to_string(),
            "M=M+1;JEQ"
        );
    }
}
