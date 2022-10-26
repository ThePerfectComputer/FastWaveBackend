// Copyright (C) 2022 Yehowshua Immanuel
// This program is distributed under both the GPLV3 license
// and the YEHOWSHUA license, both of which can be found at
// the root of the folder containing the sources for this program.
use std::fs::File;

use fastwave_backend::{ScopeIdx, VCD, parse_vcd};

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
            // for signal_idx in vcd.get_children_signal_idxs(child_scope_idx) {
            //     let signal = vcd.try_signal_idx_to_signal(signal_idx).unwrap();
            //     match signal {
            //         Signal::Data {..} => {}
            //         Signal::Alias {..} => {}
            //     }
            //     // let to_print = format!("{},{}", signal.name(), )
            // }
            // vcd.try_signal_idx_to_signal(idx)
            print_root_scope_tree(child_scope_idx, vcd.clone(), indent + 1);
        }
    }
}

fn ui_all_scopes(vcd: &VCD) {
    for root_scope_idx in vcd.root_scopes_by_idx() {
        indented_print(0, vcd.scope_name_by_idx(root_scope_idx));
        print_root_scope_tree(root_scope_idx, vcd, 1u8);
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


    let file_path = "tests/vcd-files/amaranth/up_counter.vcd";
    let file = File::open(file_path)?;
    let vcd = parse_vcd(file).unwrap();
    // let state_signal = vcd.all_si
    // for signal_idx in vcd.si
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
