// Copyright (C) 2022 Yehowshua Immanuel
// This program is distributed under both the GPLV3 license
// and the YEHOWSHUA license, both of which can be found at
// the root of the folder containing the sources for this program.
use std::fs::File;

mod files;
use files::*;

#[test]
fn parse_all_VCDs() {
    // see if we can parse all signal trees successfully
    for file_name in FILES {
        let file = File::open(file_name).unwrap();
        let vcd = fastwave_backend::parse_vcd(file);

        if !vcd.is_ok() {
            dbg!(file_name);
            vcd.unwrap();
        }
    }
}