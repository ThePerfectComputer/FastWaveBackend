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

use std::cmp::Ordering;

fn compare_strs(a: &str, b: &str) -> Ordering {
    let last_idx = if a.len() > b.len() { a.len() } else { b.len() };
    // let last_idx += -1;
    Ordering::Less
}

fn ordered_binary_lookup(map: &Vec<(String, SignalIdx)>, key: &str) -> Result<SignalIdx, String> {
    let mut upper_idx = map.len() - 1;
    let mut lower_idx = 0usize;

    while lower_idx <= upper_idx {
        let mid_idx = lower_idx + ((upper_idx - lower_idx) / 2);
        let (str_val, signal_idx) = map.get(mid_idx).unwrap();
        let ordering = key.partial_cmp(str_val.as_str()).unwrap();

        match ordering {
            Ordering::Less => {
                upper_idx = mid_idx - 1;
            }
            Ordering::Equal => {
                return Ok(*signal_idx);
            }
            Ordering::Greater => {
                lower_idx = mid_idx + 1;
            }
        }
    }

    return Err(format!(
        "Error near {}:{}. Unable to find key: `{key}` in the map.",
        file!(),
        line!()
    ));
}

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

    // the signal map should not contain any empty string
    for (k, v) in &signal_map {
        if k.len() == 0 {
            return Err(format!("Critical error near {}:{}. There should be no empty strings in vcd string -> SignalIdx hashmap.", file!(), line!()));
        }
    }

    // now that we've parsed all scopes and filled the hashmap
    // with signals, we convert hashmap to an ordered vector
    let mut signal_map1: Vec<(String, SignalIdx)> = signal_map
        .iter()
        .map(|(string, idx)| (string.clone(), idx.clone()))
        .collect();
    signal_map1.sort_by(|a: &(String, SignalIdx), b: &(String, SignalIdx)| {
        let a = &a.0;
        let b = &b.0;
        a.partial_cmp(&b).unwrap()
    });

    let now = std::time::Instant::now();
    for (k, v) in &signal_map1 {
        let signal_idx = ordered_binary_lookup(&signal_map1, k.as_str())?;
        assert!(*v == signal_idx);
    }
    let ordered_binary_search_elapsed = now.elapsed();
    println!(
        "ordered_binary_search_elapsed: {:.2?}",
        ordered_binary_search_elapsed
    );

    // parse_events(&mut wosrd_gen, &mut vcd, &mut signal_map)?;

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
