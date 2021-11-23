use std::io;
use thiserror::Error;

use crate::Label;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[error("syntax error at line {}", line)]
pub struct Error {
    line: u32,
    #[source]
    kind: ErrorKind,
}

impl Error {
    pub(crate) fn new(line: u32, kind: impl Into<ErrorKind>) -> Self {
        let kind = kind.into();
        Self { line, kind }
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("too large number: {}", _0)]
    TooLargeNumber(String),
    #[error("too far label: {}", _0)]
    TooFarLabel(String),
    #[error("invalid label statement: {}", _0)]
    InvalidLabelStatement(String),
    #[error("invalid A statement: {}", _0)]
    InvalidAStatement(String),
    #[error("invalid C statement: invalid dest: {}", _0)]
    InvalidCStatementDest(String),
    #[error("invalid C statement: invalid comp: {}", _0)]
    InvalidCStatementComp(String),
    #[error("invalid C statement: invalid jump: {}", _0)]
    InvalidCStatementJump(String),
    #[error("duplicated label: {}", _0)]
    DuplicateLabel(Label),
    #[error("too many symbols: {}", _0)]
    TooManySymbols(String),
}
