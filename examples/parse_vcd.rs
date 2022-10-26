// Copyright (C) 2022 Yehowshua Immanuel
// This program is distributed under both the GPLV3 license
// and the YEHOWSHUA license, both of which can be found at
// the root of the folder containing the sources for this program.
use clap::Parser;
use std::fs::File;

use fastwave_backend::parse_vcd;

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
    parse_vcd(file).unwrap();
    let elapsed = now.elapsed();

    println!("Parsed VCD file {} : {:.2?}", &args.path.as_os_str().to_str().unwrap(), elapsed);


    Ok(())
}
