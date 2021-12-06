use std::{
    ffi::OsStr,
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
    Open(PathBuf, #[source] io::Error),
}

impl FileReader {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, FileReaderOpenError> {
        let path = path.into();
        let file = File::open(&path).map_err(|e| FileReaderOpenError::Open(path.clone(), e))?;
        Ok(FileReader {
            path,
            file: BufReader::new(file),
        })
    }

    pub fn reader(&mut self) -> &mut BufReader<File> {
        &mut self.file
    }

    pub fn into_parts(self) -> (PathBuf, BufReader<File>) {
        (self.path, self.file)
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

#[derive(Debug)]
pub struct DirOrFileReader {
    paths: std::vec::IntoIter<PathBuf>,
}

#[derive(Debug, Error)]
pub enum DirOrFileReaderOpenError {
    #[error("failed to read directory: {}", _0.display())]
    ReadDir(PathBuf, #[source] io::Error),
    #[error("failed to read directory entry: {}", _0.display())]
    ReadDirEntry(PathBuf, #[source] io::Error),
}

impl DirOrFileReader {
    pub fn open(
        path: impl Into<PathBuf>,
        extension: impl AsRef<OsStr>,
    ) -> Result<Self, DirOrFileReaderOpenError> {
        let path = path.into();
        let paths = if path.is_dir() {
            path.read_dir()
                .map_err(|e| DirOrFileReaderOpenError::ReadDir(path.clone(), e))?
                .filter_map(|entry| {
                    entry
                        .map_err(|e| DirOrFileReaderOpenError::ReadDirEntry(path.clone(), e))
                        .map(|entry| {
                            let path = entry.path();
                            (path.is_file() && path.extension() == Some(extension.as_ref()))
                                .then(|| path)
                        })
                        .transpose()
                })
                .collect::<Result<_, _>>()?
        } else {
            vec![path]
        }
        .into_iter();
        Ok(Self { paths })
    }
}

impl Iterator for DirOrFileReader {
    type Item = Result<FileReader, FileReaderOpenError>;

    fn next(&mut self) -> Option<Self::Item> {
        let path = self.paths.next()?;
        Some(FileReader::open(path))
    }
}
