use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};
use tempfile::NamedTempFile;
use thiserror::Error;

#[derive(Debug)]
pub struct FileReader {
    path: PathBuf,
    file: BufReader<File>,
}

#[derive(Debug, Error)]
pub enum FileReaderOpenError {
    #[error("failed to open file: {}", _0.display())]
    OpenInDir(PathBuf, #[source] io::Error),
}

impl FileReader {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, FileReaderOpenError> {
        let path = path.into();
        let file =
            File::open(&path).map_err(|e| FileReaderOpenError::OpenInDir(path.clone(), e))?;
        Ok(FileReader {
            path,
            file: BufReader::new(file),
        })
    }

    pub fn reader(&mut self) -> &mut BufReader<File> {
        &mut self.file
    }
}

#[derive(Debug)]
pub struct FileWriter {
    path: PathBuf,
    dir_path: PathBuf,
    writer: BufWriter<NamedTempFile>,
}

#[derive(Debug, Error)]
pub enum FileWriterOpenError {
    #[error("no parent directory: {}", _0.display())]
    NoParentDir(PathBuf),
    #[error("failed to open file in directory: {}", _0.display())]
    OpenInDir(PathBuf, #[source] io::Error),
}

#[derive(Debug, Error)]
pub enum FileWriterPersistError {
    #[error("failed to flush file in directory: {}", _0.display())]
    Flush(PathBuf, #[source] io::Error),
    #[error("failed to persist file: {}", _0.display())]
    Persist(PathBuf, #[source] io::Error),
}

impl FileWriter {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, FileWriterOpenError> {
        let path = path.into();
        let dir_path = path
            .parent()
            .ok_or_else(|| FileWriterOpenError::NoParentDir(path.to_owned()))?;

        let file = NamedTempFile::new_in(dir_path)
            .map_err(|e| FileWriterOpenError::OpenInDir(dir_path.to_owned(), e))?;
        let writer = BufWriter::new(file);

        Ok(Self {
            path: path.to_path_buf(),
            dir_path: dir_path.to_owned(),
            writer,
        })
    }

    pub fn persist(self) -> Result<(), FileWriterPersistError> {
        self.writer
            .into_inner()
            .map_err(|e| FileWriterPersistError::Flush(self.dir_path, e.into_error()))?
            .persist(&self.path)
            .map_err(|e| FileWriterPersistError::Persist(self.path, e.error))?;
        Ok(())
    }

    pub fn writer(&mut self) -> &mut BufWriter<NamedTempFile> {
        &mut self.writer
    }
}
