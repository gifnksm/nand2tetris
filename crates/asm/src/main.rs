use color_eyre::eyre::{ensure, eyre, Context, Result};
use std::{
    env,
    fs::File,
    io::{prelude::*, BufReader, BufWriter},
    path::PathBuf,
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

    // Read input file
    let input = File::open(&input_path)
        .wrap_err_with(|| format!("failed to open file: {}", input_path.display()))?;
    let mut reader = BufReader::new(input);

    // Parse input file
    let insts = hasm::parse(&input_path, &mut reader)
        .wrap_err_with(|| format!("failed to parse file: {}", input_path.display()))?;

    // Write output file
    let output = NamedTempFile::new_in(&output_dir).wrap_err("failed to create temporary file")?;
    let mut writer = BufWriter::new(output);

    for inst in &insts {
        writeln!(&mut writer, "{:016b}", inst.encode()).wrap_err(format!(
            "failed to write output file: {}",
            output_path.display()
        ))?;
    }

    let output = writer
        .into_inner()
        .wrap_err_with(|| format!("failed to write output file: {}", output_path.display()))?;

    output
        .persist(&output_path)
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
