// Copyright (C) 2022 Yehowshua Immanuel
// This program is distributed under both the GPLV3 license
// and the YEHOWSHUA license, both of which can be found at
// the root of the folder containing the sources for this program.
use std::fs::File;

use fastwave_backend::{ScopeIdx, VCD, parse_vcd, SignalIdx};

fn indented_print(indent : u8, name : &String) {
    for _ in 0..indent {print!("  |");}
    print!("---");
    println!("{name}");
}

// TODO: refactor into more general visitor pattern that takes a 
// function as an argument.
fn visit_all_scopes(vcd: &VCD) {
    fn visit_all_scope_children(root_idx: ScopeIdx, vcd: &VCD, indent : u8) {
        if vcd.child_scopes_by_idx(root_idx).is_empty() {
        } else {
            for child_scope_idx in vcd.child_scopes_by_idx(root_idx) {
                indented_print(indent, vcd.scope_name_by_idx(child_scope_idx));
                for signal_idx in vcd.get_children_signal_idxs(child_scope_idx) {
                    let signal = vcd.signal_from_signal_idx(signal_idx);
                    let SignalIdx(idx) = signal_idx;
                    indented_print(indent + 1, &format!("{},{}", signal.name(), idx));
                }
                visit_all_scope_children(child_scope_idx, vcd.clone(), indent + 1);
            }
        }
    }
    for root_scope_idx in vcd.root_scopes_by_idx() {
        indented_print(0, vcd.scope_name_by_idx(root_scope_idx));
        visit_all_scope_children(root_scope_idx, vcd, 1u8);
    }
}

fn main() -> std::io::Result<()> {

    use std::time::Instant;
    
    // we start by printing out the entire signal tree of
    // a parsed VCD
    let now = Instant::now();
    let file_path = "tests/vcd-files/icarus/CPU.vcd";
    let file = File::open(file_path)?;
    let vcd = parse_vcd(file).unwrap();
    let elapsed = now.elapsed();
    println!("Parsed VCD file {} : {:.2?}", file_path, elapsed);

    println!("Printing Scopes");
    visit_all_scopes(&vcd);
    println!("Done Printing Scopes");
    println!();


    // we then parse another VCD, print its signal tree and 
    // query some values on its timeline
    let now = Instant::now();
    let file_path = "tests/vcd-files/amaranth/up_counter.vcd";
    let file = File::open(file_path)?;
    let vcd = parse_vcd(file).unwrap();
    let elapsed = now.elapsed();
    println!("Parsed VCD file {} : {:.2?}", file_path, elapsed);

    println!("Printing Scopes");
    visit_all_scopes(&vcd);
    println!("Done Printing Scopes");

    let state_signal = vcd.signal_from_signal_idx(SignalIdx(4));
    let name = state_signal.name();

    let timestamps = vec![31499_000u32, 31500_000u32, 57760_000u32];
    for timestamp in timestamps {
        let time = num::BigUint::from(timestamp);
        let val = state_signal
                    .query_string_val_on_tmln(&time, &vcd)
                    .unwrap();
        println!("Signal `{name}` has value `{val}` at time `{time}`");

    }

    Ok(())
}
