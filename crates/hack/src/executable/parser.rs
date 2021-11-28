use super::Executable;
use crate::{Instruction, ParseInstructionError};
use std::{
    io::{self, prelude::*},
    str::FromStr,
};
use thiserror::Error;

impl Executable {
    pub fn from_reader(mut reader: impl BufRead) -> Result<Self, ReadExecutableError> {
        let mut insts = vec![];

        let mut line_buf = String::new();
        for line in 1.. {
            line_buf.clear();
            let res = reader
                .read_line(&mut line_buf)
                .map_err(|e| ReadExecutableError::new(line, e))?;
            if res == 0 {
                break;
            }

            let inst = Instruction::from_str(line_buf.trim())
                .map_err(|e| ReadExecutableError::new(line, e))?;
            insts.push(inst);
        }

        Ok(Self { insts })
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
    InvalidInstruction(#[from] ParseInstructionError),
}
