// Copyright (C) 2022 Yehowshua Immanuel
// This program is distributed under both the GPLV3 license
// and the YEHOWSHUA license, both of which can be found at
// the root of the folder containing the sources for this program.
use std::fs::File;

use fastwave_backend::*;

use num::{BigUint};

fn indented_print(indent : u8, name : &String) {
    for _ in 0..indent {print!("  |");}
    print!("---");
    println!("{name}");
}

fn print_root_scope_tree(root_idx: ScopeIdx, vcd: &VCD, indent : u8) {
    if vcd.child_scopes_by_idx(root_idx).is_empty() {
    } else {
        for child_scope_idx in vcd.child_scopes_by_idx(root_idx) {
            indented_print(indent, vcd.scope_name_by_idx(child_scope_idx));
            let ScopeIdx(idx) = child_scope_idx;
            print_root_scope_tree(child_scope_idx, vcd.clone(), indent + 1);
        }
    }
}

fn ui_all_scopes(vcd: &VCD) {
    for root_scope_idx in vcd.root_scopes_by_idx() {
        print_root_scope_tree(root_scope_idx, vcd, 0u8);
    }
}

fn main() -> std::io::Result<()> {

    use std::time::Instant;
    
    let now = Instant::now();
    let file_path = "tests/vcd-files/icarus/CPU.vcd";
    let file = File::open(file_path)?;
    let vcd = parse_vcd(file).unwrap();
    let elapsed = now.elapsed();
    println!("Parsed VCD file {} : {:.2?}", file_path, elapsed);

    println!("Printing Scopes");
    ui_all_scopes(&vcd);


    // let state_signal = vcd.
    // let name = state_signal.name();
    // let time = BigUint::from(57760000u32);
    // let val = state_signal
    //     .query_string_val_on_tmln(
    //         &time,
    //         &vcd.tmstmps_encoded_as_u8s,
    //         &vcd.all_signals,
    //     )
    //     .unwrap();
    // println!("Signal `{name}` has value `{val}` at time `{time}`");


    Ok(())
}
