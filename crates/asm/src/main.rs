use color_eyre::eyre::{ensure, eyre, Context, Result};
use hasm::Instruction;
use std::{
    env,
    fs::File,
    io::{prelude::*, BufReader, BufWriter},
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

#[derive(Debug)]
struct Params {
    input_path: PathBuf,
    output_path: PathBuf,
    output_dir: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let Params {
        input_path,
        output_path,
        output_dir,
    } = parse_args()?;

    let reader = open_input_file(&input_path)
        .wrap_err_with(|| format!("failed to open input file: {}", input_path.display()))?;
    let insts = hasm::parse(reader)
        .wrap_err_with(|| format!("failed to parse file: {}", input_path.display()))?;
    let writer = create_output_file(&output_dir).wrap_err_with(|| {
        format!(
            "failed to create output file in directory: {}",
            &output_dir.display()
        )
    })?;
    write_output_file(&output_path, writer, &insts)
        .wrap_err_with(|| format!("failed to write output file: {}", output_path.display()))?;

    Ok(())
}

fn parse_args() -> Result<Params> {
    let args = env::args().collect::<Vec<_>>();
    ensure!(args.len() == 2, "Usage: {} <file>", args[0]);

    let input_path = PathBuf::from(&args[1]);
    let output_path = input_path.with_extension("hack");
    let output_dir = output_path
        .parent()
        .ok_or_else(|| {
            eyre!(
                "failed to get containing directory of output file: {}",
                output_path.display()
            )
        })?
        .to_owned();

    Ok(Params {
        input_path,
        output_path,
        output_dir,
    })
}

fn open_input_file(input_path: &Path) -> Result<BufReader<File>> {
    let input = File::open(input_path)
        .wrap_err_with(|| format!("failed to open file: {}", input_path.display()))?;
    let reader = BufReader::new(input);

    Ok(reader)
}

fn create_output_file(output_dir: &Path) -> Result<BufWriter<NamedTempFile>> {
    let output = NamedTempFile::new_in(&output_dir)?;
    Ok(BufWriter::new(output))
}

fn write_output_file(
    output_path: &Path,
    mut writer: BufWriter<NamedTempFile>,
    insts: &[Instruction],
) -> Result<()> {
    for inst in insts {
        writeln!(&mut writer, "{:016b}", inst.encode())?;
    }

    writer.into_inner()?.persist(&output_path)?;

    Ok(())
}
