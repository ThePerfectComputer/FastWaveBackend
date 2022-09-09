use clap::Parser;
use std::fs::File;

use fastwave::*;

use num::{BigUint};

fn main() -> std::io::Result<()> {

    use std::time::Instant;
    
    let now = Instant::now();
    let file_path = "tests/vcd-files/amaranth/up_counter.vcd";
    let file = File::open(file_path)?;
    let vcd = parse_vcd(file).unwrap();
    let elapsed = now.elapsed();

    println!("Parsed VCD file {} : {:.2?}", file_path, elapsed);

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


    Ok(())
}
