use color_eyre::eyre::{ensure, Context, Result};
use common::fs::FileWriter;
use std::{env, io::prelude::*, path::PathBuf};
use vm::{asm::Statement, Executable};

#[derive(Debug)]
struct Params {
    input_paths: Vec<PathBuf>,
    output_path: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let Params {
        input_paths,
        output_path,
    } = parse_args()?;

    let exec = Executable::open(&input_paths).wrap_err("failed to open executable")?;
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

    let input_paths = if input_path.is_dir() {
        input_path
            .read_dir()
            .wrap_err_with(|| format!("failed to read directory: {}", input_path.display()))?
            .filter_map(|entry| {
                entry
                    .wrap_err_with(|| {
                        format!("failed to read directory entry: {}", input_path.display())
                    })
                    .map(|entry| {
                        let path = entry.path();
                        (path.is_file() && path.extension() == Some("vm".as_ref())).then(|| path)
                    })
                    .transpose()
            })
            .collect::<Result<_>>()?
    } else {
        vec![input_path]
    };

    Ok(Params {
        input_paths,
        output_path,
    })
}

fn write_output_file(mut writer: impl Write, insts: &[Statement]) -> Result<()> {
    for inst in insts {
        writeln!(writer, "{}", inst)?;
    }
    Ok(())
}
