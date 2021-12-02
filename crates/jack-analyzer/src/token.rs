use crate::{xml::XmlEscape, Error};
use common::fs::FileWriter;
use jack::{Token, TokenKind};
use std::{io::prelude::*, path::PathBuf};

#[derive(Debug)]
pub(crate) struct TokenWriter {
    path: PathBuf,
    writer: FileWriter,
}

impl TokenWriter {
    pub(crate) fn open(path: PathBuf) -> Result<Self, Error> {
        let mut writer = FileWriter::open(&path)
            .map_err(|e| Error::CreateOutputFile(path.to_owned(), e.into()))?;

        writeln!(writer.writer(), "<tokens>")
            .map_err(|e| Error::WriteToken(path.clone(), e.into()))?;

        Ok(Self { path, writer })
    }

    pub(crate) fn write(&mut self, token: &Token) -> Result<(), Error> {
        let tag = match token.kind() {
            TokenKind::Keyword => "keyword",
            TokenKind::Symbol => "symbol",
            TokenKind::Int => "integerConstant",
            TokenKind::String => "stringConstant",
            TokenKind::Ident => "identifier",
        };
        let value = token.to_cow_str();
        writeln!(
            self.writer.writer(),
            "<{tag}> {} </{tag}>",
            XmlEscape(&value),
            tag = tag
        )
        .map_err(|e| Error::WriteToken(self.path.to_owned(), e.into()))?;
        Ok(())
    }

    pub(crate) fn persist(mut self) -> Result<(), Error> {
        writeln!(self.writer.writer(), "</tokens>")
            .map_err(|e| Error::WriteToken(self.path.to_owned(), e.into()))?;
        self.writer
            .persist()
            .map_err(|e| Error::PersistOutputFile(self.path, e.into()))?;
        Ok(())
    }
}
