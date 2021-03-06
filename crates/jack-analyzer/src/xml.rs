use crate::Error;
use common::fs::FileWriter;
use jack::token::WithLoc;
use std::{
    fmt,
    io::{self, prelude::*},
    path::PathBuf,
};

#[derive(Debug)]
pub(crate) struct XmlEscape<'a>(pub &'a str);

impl fmt::Display for XmlEscape<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = self.0;
        while !s.is_empty() {
            if let Some(idx) = s.find(['&', '<', '>', '"', '\''].as_ref()) {
                let (l, r) = s.split_at(idx);
                write!(f, "{}", l)?;
                s = r;
                let mut cs = s.chars();
                match cs.next() {
                    Some('&') => write!(f, "&amp;")?,
                    Some('<') => write!(f, "&lt;")?,
                    Some('>') => write!(f, "&gt;")?,
                    Some('"') => write!(f, "&quot;")?,
                    Some('\'') => write!(f, "&apos;")?,
                    _ => unreachable!(),
                }
                s = cs.as_str();
                continue;
            }
            write!(f, "{}", s)?;
            break;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct XmlWriter {
    path: PathBuf,
    writer: FileWriter,
}

impl XmlWriter {
    pub(crate) fn open(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path = path.into();
        let writer = FileWriter::open(&path)
            .map_err(|e| Error::CreateOutputFile(path.to_owned(), e.into()))?;
        Ok(Self { path, writer })
    }

    pub(crate) fn write(&mut self, class: &impl WriteXml) -> Result<(), Error> {
        class
            .write_xml(0, self)
            .map_err(|e| Error::WriteXml(self.path.to_owned(), e.into()))?;
        Ok(())
    }

    pub(crate) fn persist(self) -> Result<(), Error> {
        self.writer
            .persist()
            .map_err(|e| Error::PersistOutputFile(self.path, e.into()))?;
        Ok(())
    }

    pub(crate) fn write_open(&mut self, indent: usize, tag: &str) -> io::Result<()> {
        writeln!(
            self.writer.writer(),
            "{:indent$}<{tag}>",
            "",
            tag = tag,
            indent = indent * 2
        )?;
        Ok(())
    }

    pub(crate) fn write_close(&mut self, indent: usize, tag: &str) -> io::Result<()> {
        writeln!(
            self.writer.writer(),
            "{:indent$}</{tag}>",
            "",
            tag = tag,
            indent = indent * 2
        )?;
        Ok(())
    }

    pub(crate) fn write_multi(
        &mut self,
        indent: usize,
        tag: &str,
        mut f: impl FnMut(usize, &mut Self) -> io::Result<()>,
    ) -> io::Result<()> {
        self.write_open(indent, tag)?;
        f(indent + 1, self)?;
        self.write_close(indent, tag)?;
        Ok(())
    }

    pub(crate) fn write_single(&mut self, indent: usize, tag: &str, value: &str) -> io::Result<()> {
        writeln!(
            self.writer.writer(),
            "{:indent$}<{tag}> {} </{tag}>",
            "",
            XmlEscape(value),
            tag = tag,
            indent = indent * 2
        )?;
        Ok(())
    }

    pub(crate) fn write_labeled(
        &mut self,
        indent: usize,
        tag: &str,
        item: &impl WriteXml,
    ) -> io::Result<()> {
        self.write_multi(indent, tag, |indent, writer| item.write_xml(indent, writer))
    }

    pub(crate) fn write_opt(
        &mut self,
        indent: usize,
        tag: &str,
        item: &Option<impl WriteXml>,
    ) -> io::Result<()> {
        if let Some(item) = item {
            self.write_multi(indent, tag, |indent, writer| item.write_xml(indent, writer))?;
        }
        Ok(())
    }

    pub(crate) fn write_list(
        &mut self,
        indent: usize,
        tag: &str,
        list: &[impl WriteXml],
    ) -> io::Result<()> {
        self.write_multi(indent, tag, |indent, writer| {
            for item in list {
                item.write_xml(indent, writer)?;
            }
            Ok(())
        })
    }

    pub(crate) fn write_list_with_sep(
        &mut self,
        indent: usize,
        tag: &str,
        list: &[impl WriteXml],
        sep: impl WriteXml,
    ) -> io::Result<()> {
        self.write_multi(indent, tag, |indent, writer| {
            for (i, item) in list.iter().enumerate() {
                if i > 0 {
                    sep.write_xml(indent, writer)?;
                }
                item.write_xml(indent, writer)?;
            }
            Ok(())
        })
    }
}

pub(crate) trait WriteXml {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()>;
}

impl<T> WriteXml for WithLoc<T>
where
    T: WriteXml,
{
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        self.data.write_xml(indent, writer)
    }
}
