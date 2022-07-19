use std::{fs::File};
use std::collections::HashMap;
use num::BigInt;
use num::bigint::ToBigInt;

use super::*;

mod combinator_atoms;
use combinator_atoms::*;

mod types;
use types::*;

mod metadata;
use metadata::*;

mod scopes;
use scopes::*;

use function_name::named;

#[named]
fn parse_events<'a>(
    word_reader      : &mut WordReader,
    vcd              : &'a mut VCD,
    signal_map       : &mut HashMap<String, Signal_Idx>
) -> Result<(), String> {

    loop {
        let next_word = word_reader.next_word();
        // if we've reached the end of the file, then there is obviously
        // nothing left to do...
        if next_word.is_none() {break};

        let (word, cursor) = next_word.unwrap();
        match &word[0..1] {
            "$" => {continue}
            "#" => {continue}
            _ => {}
        }
    }

    Ok(())
}

pub fn parse_vcd(file : File) -> Result<VCD, String> {
    let mut word_gen = WordReader::new(file);

    let header = parse_metadata(&mut word_gen)?;

    let mut signal_map = std::collections::HashMap::new();

    let mut vcd = VCD{
        metadata   : header,
        cursor     : 0.to_bigint().unwrap(),
        all_signals: vec![],
        all_scopes : vec![],
        scope_roots: vec![],
    };

    parse_scopes(&mut word_gen, None, &mut vcd, &mut signal_map)?;
    // parse_events(&mut word_gen, &mut vcd, &mut signal_map)?;

    Ok(vcd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use std::fs::File;
    #[test]
    fn headers() {
        // TODO: eventually, once all dates pass, merge the following
        // two loops
        // testing dates
        for file in test::good_date_files {
            let metadata = parse_metadata(
                &mut WordReader::new(
                    File::open(file)
                    .unwrap()
                )
            );
            assert!(metadata.is_ok());
            assert!(metadata.unwrap().date.is_some());
        }

        for file in test::files {
            let metadata = parse_metadata(
                &mut WordReader::new(
                    File::open(file)
                    .unwrap()
                )
            );
            assert!(metadata.is_ok());

            let (scalar, timescale) = metadata.unwrap().timescale;
            assert!(scalar.is_some());
        }

    }

    #[test]
    fn scopes() {
        // see if we can parse all signal trees successfully
        for file_name in test::files {
            let file = File::open(file_name).unwrap();
            let vcd = parse_vcd(file);

            if !vcd.is_ok() {
                dbg!(file_name);
                vcd.unwrap();
            }

            // assert!(vcd.is_ok());
        }

    }
}