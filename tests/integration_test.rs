use std::fs::File;

mod files;
use files::*;

#[test]
fn parse_all_VCDs() {
    // see if we can parse all signal trees successfully
    for file_name in FILES {
        let file = File::open(file_name).unwrap();
        let vcd = fastwave::parse_vcd(file);

        if !vcd.is_ok() {
            dbg!(file_name);
            vcd.unwrap();
        }
    }
}