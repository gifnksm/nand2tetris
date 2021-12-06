use color_eyre::eyre::{ensure, Context, Result};
use common::{
    fs::{DirOrFileReader, FileWriter},
    iter::TryIterator,
};
use std::{env, io::prelude::*, path::PathBuf};
use vm::{asm::Statement, Executable};

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

    let files = DirOrFileReader::open(&input_path, "vm")
        .wrap_err_with(|| format!("failed to open input file: {}", input_path.display()))?;

    let input_modules = files
        .map_ok(|file| file.into_parts())
        .collect::<Result<Vec<_>, _>>()
        .wrap_err_with(|| format!("failed to open input file: {}", input_path.display()))?;

    let exec = Executable::from_readers(input_modules).wrap_err("failed to open executable")?;
    let stmts = exec.translate();

    let mut writer = FileWriter::open(&output_path)
        .wrap_err_with(|| format!("failed to create output file: {}", output_path.display()))?;
    write_output_file(writer.writer(), &stmts)
        .wrap_err_with(|| format!("failed to write output file: {}", output_path.display()))?;
    writer
        .persist()
        .wrap_err_with(|| format!("failed to persist output file: {}", output_path.display()))?;

    Ok(())
}

fn parse_args() -> Result<Params> {
    let args = env::args().collect::<Vec<_>>();
    ensure!(args.len() == 2, "Usage: {} <file>", args[0]);
    create_params(PathBuf::from(&args[1]), None)
}

fn create_params(input_path: PathBuf, output_path: Option<PathBuf>) -> Result<Params> {
    let output_path = output_path.unwrap_or_else(|| {
        if input_path.is_dir() {
            let mut output_name = input_path
                .components()
                .last()
                .unwrap()
                .as_os_str()
                .to_owned();
            output_name.push(".asm");
            input_path.join(output_name)
        } else {
            input_path.with_extension("asm")
        }
    });

    Ok(Params {
        input_path,
        output_path,
    })
}

fn write_output_file(mut writer: impl Write, insts: &[Statement]) -> Result<()> {
    for inst in insts {
        writeln!(writer, "{}", inst)?;
    }
    Ok(())
}
