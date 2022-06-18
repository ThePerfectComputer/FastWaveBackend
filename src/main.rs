use std::fs::File;
use clap::Parser;

pub mod vcd;
use vcd::*;

#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let file = File::open(&args.path)?;
    parse_vcd(file);

    Ok(())
}