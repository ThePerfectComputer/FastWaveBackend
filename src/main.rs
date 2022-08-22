use clap::Parser;
use std::fs::File;

pub mod test;

pub mod vcd;
use vcd::parse_vcd;

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
    println!("Elapsed: {:.2?}", elapsed);

    // std::thread::sleep(std::time::Duration::from_secs(10));

    Ok(())
}
