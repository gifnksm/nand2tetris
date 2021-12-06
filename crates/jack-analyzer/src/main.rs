use crate::xml::XmlWriter;
use color_eyre::eyre::{ensure, Result};
use common::{
    fs::FileReader,
    iter::{IteratorExt, TryIterator},
};
use jack::{Class, FromTokens, Tokens};
use std::{env, path::PathBuf};
use thiserror::Error;
use token::TokenWriter;

mod ast;
mod token;
mod xml;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to open input file: {}", _0.display())]
    OpenInputFile(PathBuf, #[source] StdError),
    #[error("failed to create output file: {}", _0.display())]
    CreateOutputFile(PathBuf, #[source] StdError),
    #[error("failed to read token from file: {}", _0.display())]
    ReadToken(PathBuf, #[source] StdError),
    #[error("failed to parse file: {}", _0.display())]
    Parse(PathBuf, #[source] StdError),
    #[error("failed to write token to file: {}", _0.display())]
    WriteToken(PathBuf, #[source] StdError),
    #[error("failed to write AST to file: {}", _0.display())]
    WriteAst(PathBuf, #[source] StdError),
    #[error("failed to persist output file: {}", _0.display())]
    PersistOutputFile(PathBuf, #[source] StdError),
}

type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
struct Params {
    input_path: PathBuf,
    token_output_path: PathBuf,
    ast_output_path: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let Params {
        input_path,
        token_output_path,
        ast_output_path,
    } = parse_args()?;

    let mut reader = FileReader::open(&input_path)?;

    let mut token_writer = TokenWriter::open(token_output_path)?;
    let mut ast_writer = XmlWriter::open(ast_output_path)?;

    let tokens = Tokens::from_reader(reader.reader())
        .map(|res| res.map_err(|e| Error::ReadToken(input_path.to_owned(), e.into())))
        .try_inspect_ok(|token| token_writer.write(&token.data));

    let ast = Class::from_tokens(&mut tokens.prependable())
        .map_err(|e| Error::Parse(input_path.to_owned(), e.into()))?;
    ast_writer.write(&ast)?;

    token_writer.persist()?;
    ast_writer.persist()?;

    Ok(())
}

fn parse_args() -> Result<Params> {
    let args = env::args().collect::<Vec<_>>();
    ensure!(args.len() == 2, "Usage: {} <file>", args[0]);

    let input_path = PathBuf::from(&args[1]);
    let token_output_path = input_path.with_extension("token.xml");
    let ast_output_path = input_path.with_extension("ast.xml");

    Ok(Params {
        input_path,
        token_output_path,
        ast_output_path,
    })
}
