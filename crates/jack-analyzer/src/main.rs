use crate::xml::XmlWriter;
use color_eyre::eyre::{ensure, Result};
use common::{
    fs::{DirOrFileReader, FileWriter},
    iter::{IteratorExt, TryIterator},
};
use jack::{
    ast::{Class, FromTokens},
    symbol_table::GlobalSymbolTable,
    token::Tokens,
    typed_ast::ToControlFlowGraph,
};
use std::{env, io::prelude::*, path::PathBuf};
use thiserror::Error;
use token::TokenWriter;

mod ast;
mod control_flow_graph;
mod token;
mod typed_ast;
mod xml;

type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;

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
    #[error("failed to register symbols from file: {}", _0.display())]
    ExtendSymbolTable(PathBuf, #[source] StdError),
    #[error("failed to write xml to file: {}", _0.display())]
    WriteXml(PathBuf, #[source] StdError),
    #[error("failed to persist output file: {}", _0.display())]
    PersistOutputFile(PathBuf, #[source] StdError),
    #[error("failed to resolve symbols in file: {}", _0.display())]
    Resolve(PathBuf, #[source] StdError),
    #[error("failed to convert to control flow graph file: {}", _0.display())]
    ToCfg(PathBuf, #[source] StdError),
    #[error("failed to optimize to control flow graph file: {}", _0.display())]
    Optimize(PathBuf, #[source] StdError),
    #[error("failed to write VM command to file: {}", _0.display())]
    WriteVmCommand(PathBuf, #[source] StdError),
}

#[derive(Debug)]
struct Params {
    input_path: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let Params { input_path } = parse_args()?;

    let mut symbol_table = GlobalSymbolTable::with_builtin();
    let mut asts = vec![];
    let mut token_writers = vec![];
    let mut xml_writers = vec![];
    let mut file_writers = vec![];

    DirOrFileReader::open(&input_path, "jack")?.try_for_each(|reader| {
        let (input_path, reader) = reader
            .map_err(|e| Error::OpenInputFile(input_path.clone(), e.into()))?
            .into_parts();
        let token_output_path = input_path.with_extension("token.xml");
        let ast_output_path = input_path.with_extension("ast.xml");

        let mut token_writer = TokenWriter::open(token_output_path)?;
        let mut ast_writer = XmlWriter::open(ast_output_path)?;

        let tokens = Tokens::from_reader(reader)
            .map(|res| res.map_err(|e| Error::ReadToken(input_path.to_owned(), e.into())))
            .try_inspect_ok(|token| token_writer.write(&token.data));

        let ast = Class::from_tokens(&mut tokens.prependable())
            .map_err(|e| Error::Parse(input_path.to_owned(), e.into()))?;
        ast_writer.write(&ast)?;

        symbol_table
            .extend_with_class(&input_path, &ast.data)
            .map_err(|e| Error::ExtendSymbolTable(input_path.to_owned(), e.into()))?;

        asts.push((input_path, ast));
        token_writers.push(token_writer);
        xml_writers.push(ast_writer);
        Ok::<(), Error>(())
    })?;

    for (input_path, ast) in asts {
        let typed_ast_output_path = input_path.with_extension("typed-ast.xml");
        let mut typed_ast_writer = XmlWriter::open(&typed_ast_output_path)?;
        let typed_ast = ast
            .resolve(&symbol_table)
            .map_err(|e| Error::Resolve(input_path.clone(), e.into()))?;
        typed_ast_writer.write(&typed_ast)?;
        xml_writers.push(typed_ast_writer);

        let cfg_output_path = input_path.with_extension("cfg.xml");
        let mut cfg_writer = XmlWriter::open(&cfg_output_path)?;
        let mut cfg = typed_ast
            .to_control_flow_graph()
            .map_err(|e| Error::ToCfg(input_path.clone(), e.into()))?;
        cfg_writer.write(&cfg)?;
        xml_writers.push(cfg_writer);

        let cfg_opt_output_path = input_path.with_extension("cfg-optimized.xml");
        let mut cfg_opt_writer = XmlWriter::open(&cfg_opt_output_path)?;
        cfg.optimize()
            .map_err(|e| Error::Optimize(input_path.clone(), e.into()))?;
        cfg_opt_writer.write(&cfg)?;
        xml_writers.push(cfg_opt_writer);

        let vm_output_path = input_path.with_extension("vm");
        let mut vm_writer = FileWriter::open(&vm_output_path)?;
        let commands = cfg.to_vm();
        for command in &commands {
            writeln!(vm_writer.writer(), "{}", command)
                .map_err(|e| Error::WriteVmCommand(input_path.clone(), e.into()))?;
        }
        file_writers.push(vm_writer);
    }

    for token_writer in token_writers {
        token_writer.persist()?;
    }
    for xml_writer in xml_writers {
        xml_writer.persist()?;
    }
    for file_writer in file_writers {
        file_writer.persist()?;
    }

    Ok(())
}

fn parse_args() -> Result<Params> {
    let args = env::args().collect::<Vec<_>>();
    ensure!(args.len() == 2, "Usage: {} <file>", args[0]);
    let input_path = PathBuf::from(&args[1]);
    Ok(Params { input_path })
}
