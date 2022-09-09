use clap::Parser;
use std::fs::File;

use fastwave::*;

use num::{BigUint};

#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    use std::time::Instant;
    
    let now = Instant::now();
    let file = File::open(&args.path)?;
    let vcd = parse_vcd(file).unwrap();
    let elapsed = now.elapsed();

    println!("Parsed VCD file {} : {:.2?}", &args.path.as_os_str().to_str().unwrap(), elapsed);


    Ok(())
}
