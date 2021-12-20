use crate::{
    xml::{WriteXml, XmlWriter},
    Error,
};
use jack::token::{Ident, Keyword, Symbol, Token};
use std::{io, path::PathBuf};

#[derive(Debug)]
pub(crate) struct TokenWriter {
    path: PathBuf,
    writer: XmlWriter,
}

impl TokenWriter {
    pub(crate) fn open(path: PathBuf) -> Result<Self, Error> {
        let mut writer = XmlWriter::open(path.clone())
            .map_err(|e| Error::CreateOutputFile(path.to_owned(), e.into()))?;

        writer
            .write_open(0, "tokens")
            .map_err(|e| Error::WriteXml(path.clone(), e.into()))?;

        Ok(Self { path, writer })
    }

    pub(crate) fn write(&mut self, token: &Token) -> Result<(), Error> {
        token
            .write_xml(0, &mut self.writer)
            .map_err(|e| Error::WriteXml(self.path.clone(), e.into()))?;
        Ok(())
    }

    pub(crate) fn persist(mut self) -> Result<(), Error> {
        self.writer
            .write_close(0, "tokens")
            .map_err(|e| Error::WriteXml(self.path.clone(), e.into()))?;
        self.writer.persist()?;
        Ok(())
    }
}

impl WriteXml for Token {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        match self {
            Token::Keyword(kw) => kw.write_xml(indent, writer),
            Token::Symbol(sym) => sym.write_xml(indent, writer),
            Token::Int(n) => n.write_xml(indent, writer),
            Token::String(s) => s.write_xml(indent, writer),
            Token::Ident(ident) => ident.write_xml(indent, writer),
        }
    }
}

impl WriteXml for Keyword {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_single(indent, "keyword", self.as_str())
    }
}

impl WriteXml for Symbol {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_single(indent, "symbol", self.as_str())
    }
}

impl WriteXml for u16 {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_single(indent, "integerConstant", &self.to_string())
    }
}

impl WriteXml for String {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_single(indent, "stringConstant", self.as_str())
    }
}

impl WriteXml for Ident {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_single(indent, "identifier", self.as_str())
    }
}
