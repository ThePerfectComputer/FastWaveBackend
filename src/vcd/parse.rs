use num::BigInt;
use std::collections::HashMap;
use std::fs::File;

use super::*;

mod combinator_atoms;
use combinator_atoms::*;

mod types;
use types::*;

mod metadata;
use metadata::*;

mod scopes;
use scopes::*;

mod events;
use events::*;

pub fn parse_vcd(file: File) -> Result<VCD, String> {
    let mut word_gen = WordReader::new(file);

    let header = parse_metadata(&mut word_gen)?;

    // later, we'll need to map parsed ascii symbols to their
    // respective signal indexes
    let mut signal_map = std::collections::HashMap::new();

    // after we parse metadata, we form the VCD object
    let mut vcd = VCD {
        metadata: header,
        timeline: vec![],
        timeline_markers: vec![],
        all_signals: vec![],
        all_scopes: vec![],
        scope_roots: vec![],
    };

    parse_scopes(&mut word_gen, &mut vcd, &mut signal_map)?;
    parse_events(&mut word_gen, &mut vcd, &mut signal_map)?;

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
        for file in test::GOOD_DATE_FILES {
            let metadata = parse_metadata(&mut WordReader::new(File::open(file).unwrap()));
            assert!(metadata.is_ok());
            assert!(metadata.unwrap().date.is_some());
        }

        for file in test::FILES {
            let metadata = parse_metadata(&mut WordReader::new(File::open(file).unwrap()));
            assert!(metadata.is_ok());

            let (scalar, _timescale) = metadata.unwrap().timescale;
            assert!(scalar.is_some());
        }
    }

    #[test]
    fn scopes() {
        // see if we can parse all signal trees successfully
        for file_name in test::FILES {
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
