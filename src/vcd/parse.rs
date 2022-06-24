use chrono::prelude::*;
use itertools::Itertools;
use std::fs::File;
use ::function_name::named;

use super::*;

mod combinator_atoms;
use combinator_atoms::*;

mod types;
use types::*;

mod metadata;
use metadata::*;


pub fn parse_vcd(file : File) {
    let mut word_gen = WordReader::new(file);

    let header = parse_metadata(&mut word_gen).unwrap();
    dbg!(header);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use std::fs::File;
    #[test]
    fn headers() {
        for file in test::files {
            let metadata = parse_metadata(
                &mut WordReader::new(
                    File::open(file)
                    .unwrap()
                )
            );
            assert!(metadata.is_ok());
            assert!(metadata.unwrap().date.is_some());
        }

    }
}