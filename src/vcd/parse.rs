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

// use function_name::named;

#[named]
fn parse_signal_tree<'a>(
    word_reader : &mut WordReader,
    vcd         : &'a mut VCD
) -> Result<&'a mut VCD, String> {
    let err : Result<&'a mut VCD, String> = Err(format!("reached end of file without parser leaving {}", function_name!()));
    // we assume we've already seen a `$scope` once
    // by the time we reach this function
    // let scope_name = 
    loop {
        let word = word_reader.next_word();

        // if there isn't another word left in the file, then we exit
        if word.is_none() {
            return err;
        }
    }
    Ok(vcd)
}


pub fn parse_vcd(file : File) {
    let mut word_gen = WordReader::new(file);

    let header = parse_metadata(&mut word_gen).unwrap();
    dbg!(&header);

    // let (word, cursor) = word_gen.next_word().unwrap();
    // cursor.error(word).unwrap();

    let mut vcd = VCD{
        metadata: header,
        all_signals: vec![],
        all_scopes: vec![]
    };
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
}