use clap::Parser;
use std::fs::File;

use fastwave::*;

use num::{BigUint};

fn main() -> std::io::Result<()> {

    use std::time::Instant;
    
    let now = Instant::now();
    let file_path = "tests/vcd-files/icarus/CPU.vcd";
    let file = File::open(file_path).unwrap();
    let vcd = parse_vcd(file).unwrap();
    let elapsed = now.elapsed();

    println!("Parsed VCD file {} : {:.2?}", file_path, elapsed);

    // testbench -> CPU -> rs2_data[31:0] @ 4687s
    let rs2_data_signal = &vcd.all_signals[51];
    let name = rs2_data_signal.name();
    let time = BigUint::from(4687u32);
    let val = rs2_data_signal
        .query_num_val_on_tmln(
            &time,
            &vcd.tmstmps_encoded_as_u8s,
            &vcd.all_signals,
        )
        .unwrap();
    println!("Signal `{name}` has value `{val}` at time `{time}`");

    Ok(())
}
