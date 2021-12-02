use asm::{hack::Instruction, Executable};
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

    let exec = Executable::from_reader(reader.reader())
        .wrap_err_with(|| format!("failed to parse file: {}", input_path.display()))?;
    let insts = exec
        .assemble()
        .wrap_err_with(|| format!("failed to assemble file: {}", input_path.display()))?;

    let mut writer = FileWriter::open(&output_path)
        .wrap_err_with(|| format!("failed to create output file: {}", output_path.display()))?;
    write_output_file(writer.writer(), &insts)
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
    let output_path = input_path.with_extension("hack");

    Ok(Params {
        input_path,
        output_path,
    })
}

fn write_output_file(mut writer: impl Write, insts: &[Instruction]) -> Result<()> {
    for inst in insts {
        writeln!(writer, "{:016b}", inst.encode())?;
    }
    Ok(())
}
