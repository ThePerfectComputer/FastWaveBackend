use clap::Parser;
use std::fs::File;

pub mod test;

pub mod vcd;
use vcd::*;

use num::BigUint;

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
    println!("Elapsed: {:.2?}", elapsed);

    // the following is really only for test-vcd-files/icarus/CPU.vcd
    // at the moment
    if args.path.as_os_str().to_str().unwrap() == "test-vcd-files/icarus/CPU.vcd" {
        let signal = &vcd.all_signals[51];
        let name = match signal {
            Signal::Data { name, .. } => name,
            _ => "ERROR",
        };
        let val = signal
            .query_num_val_on_tmln(
                BigUint::from(4687u32),
                &vcd.tmstmps_encoded_as_u8s,
                &vcd.all_signals,
            )
            .unwrap();
        dbg!(format!("{val:#X}"));
        dbg!(name);
    }

    Ok(())
}
