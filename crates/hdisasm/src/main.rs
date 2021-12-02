use asm::{hack, Statement};
use color_eyre::eyre::{ensure, Context, Result};
use common::fs::{FileReader, FileWriter};
use std::{env, io::prelude::*, path::PathBuf};

#[derive(Debug)]
struct Params {
    input_path: PathBuf,
    output_path: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let Params {
        input_path,
        output_path,
    } = parse_args()?;

    let mut reader = FileReader::open(&input_path)
        .wrap_err_with(|| format!("failed to open input file: {}", input_path.display()))?;

    let exec = hack::Executable::from_reader(reader.reader())
        .wrap_err_with(|| format!("failed to parse file: {}", input_path.display()))?;

    let exec = asm::Executable::disassemble(exec);
    let stmts = exec.statements();

    let mut writer = FileWriter::open(&output_path)
        .wrap_err_with(|| format!("failed to create output file: {}", output_path.display()))?;
    write_output_file(writer.writer(), stmts)
        .wrap_err_with(|| format!("failed to write output file: {}", output_path.display()))?;
    writer
        .persist()
        .wrap_err_with(|| format!("failed to persist output file: {}", output_path.display()))?;

    Ok(())
}

fn parse_args() -> Result<Params> {
    let args = env::args().collect::<Vec<_>>();
    ensure!(args.len() == 2, "Usage: {} <file>", args[0]);

    let input_path = PathBuf::from(&args[1]);
    let output_path = input_path.with_extension("dasm");

    Ok(Params {
        input_path,
        output_path,
    })
}

fn write_output_file(mut writer: impl Write, stmts: &[Statement]) -> Result<()> {
    for stmt in stmts {
        writeln!(writer, "{}", stmt)?;
    }
    Ok(())
}
