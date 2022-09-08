use clap::Parser;
use std::fs::File;

pub mod test;

pub mod vcd;
use vcd::*;

use num::{BigUint, traits::sign};

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

    // the following is really only for test-vcd-files/icarus/CPU.vcd
    // at the moment
    if args.path.as_os_str().to_str().unwrap() == "test-vcd-files/icarus/CPU.vcd" {
        let rs2_data_signal = &vcd.all_signals[51];
        let name = rs2_data_signal.name();
        // query testbench -> CPU -> rs2_data[31:0] @ 4687s
        let time = BigUint::from(4687u32);
        let val = rs2_data_signal
            .query_num_val_on_tmln(
                &time,
                &vcd.tmstmps_encoded_as_u8s,
                &vcd.all_signals,
            )
            .unwrap();
        println!("Signal `{name}` has value `{val}` at time `{time}`");
    
    // also need to test testbench -> CPU -> ID_EX_RD[4:0]
    }

    // this is to help with testing stringed enums
    if args.path.as_os_str().to_str().unwrap() == "test-vcd-files/amaranth/up_counter.vcd" {
        let state_signal = &vcd.all_signals[4];
        let name = state_signal.name();
        let time = BigUint::from(57760000u32);
        let val = state_signal
            .query_string_val_on_tmln(
                &time,
                &vcd.tmstmps_encoded_as_u8s,
                &vcd.all_signals,
            )
            .unwrap();
        println!("Signal `{name}` has value `{val}` at time `{time}`");
    }


    Ok(())
}
