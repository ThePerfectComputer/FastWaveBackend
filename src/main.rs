use std::fs::File;
use clap::Parser;

pub mod test;
use test::*;

pub mod vcd;
use vcd::parse_vcd;

#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let file = File::open(&args.path)?;
    let vcd = parse_vcd(file).unwrap(); 

    // println!("printing signal tree");
    // vcd.print_scopes();

    Ok(())
}