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
        let Cursor(Line(_), Word(word_in_line_idx)) = cursor;
        // we only want to match on the first word in a line
        if word_in_line_idx != 1 {continue}
        match &word[0..1] {
            "$" => {}
            "#" => {
                let value = &word[1..];
                let time_cursor = BigInt::parse_bytes(value.as_bytes(), 10).ok_or(
                    format!("failed to parse {value} as BigInt at {cursor:?}").as_str())?;
                vcd.cursor = time_cursor;
            }
            "0" => {
                // lokup signal idx
                let hash = &word[1..].to_string();
                let Signal_Idx(ref signal_idx) = signal_map.get(hash).ok_or(
                    format!("failed to lookup signal {hash} at {cursor:?}").as_str())?;

                // account for fact that signal idx could be an alias, so there
                // could be one step of indirection
                let signal_idx = 
                {
                    let signal = vcd.all_signals.get(*signal_idx).unwrap();
                    match signal {
                    Signal::Data {..} => {signal_idx.clone()}
                    Signal::Alias {name, signal_alias} => {
                            let Signal_Idx(ref signal_idx) = signal_alias;
                            signal_idx.clone()

                        }
                    }
                };

                // after handling potential indirection, go ahead and update the timeline
                // of the signal signal_idx references
                let signal = vcd.all_signals.get_mut(signal_idx).unwrap();
                match signal {
                    Signal::Data {name, sig_type, num_bits, 
                    self_idx, timeline, scope_parent} => {
                        let value = 0.to_bigint().unwrap();
                        let pair = (TimeStamp(vcd.cursor.clone()), Sig_Value::Numeric(value));
                        timeline.push(pair);
                        Ok(())
                    }
                    Signal::Alias {..} => {
                        let (f, l )= (file!(), line!());
                        let msg = format!(
                            "Error near {f}:{l}, a signal alias should not point to a signal alias.\n\
                                This error occurred while parsing vcd file at {cursor:?}");
                        Err(msg)
                    }
                }?;
            }
            "1" => {
                // lokup signal idx
                let hash = &word[1..].to_string();
                let Signal_Idx(ref signal_idx) = signal_map.get(hash).ok_or(
                    format!("failed to lookup signal {hash} at {cursor:?}").as_str())?;

                // account for fact that signal idx could be an alias, so there
                // could be one step of indirection
                let signal_idx = 
                {
                    let signal = vcd.all_signals.get(*signal_idx).unwrap();
                    match signal {
                    Signal::Data {..} => {signal_idx.clone()}
                    Signal::Alias {name, signal_alias} => {
                            let Signal_Idx(ref signal_idx) = signal_alias;
                            signal_idx.clone()

                        }
                    }
                };

                // after handling potential indirection, go ahead and update the timeline
                // of the signal signal_idx references
                let signal = vcd.all_signals.get_mut(signal_idx).unwrap();
                match signal {
                    Signal::Data {name, sig_type, num_bits, 
                    self_idx, timeline, scope_parent} => {
                        let value = 1.to_bigint().unwrap();
                        let pair = (TimeStamp(vcd.cursor.clone()), Sig_Value::Numeric(value));
                        timeline.push(pair);
                        Ok(())
                    }
                    Signal::Alias {..} => {
                        let (f, l )= (file!(), line!());
                        let msg = format!(
                            "Error near {f}:{l}, a signal alias should not point to a signal alias.\n\
                                This error occurred while parsing vcd file at {cursor:?}");
                        Err(msg)
                    }
                }?;
            }
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
    parse_events(&mut word_gen, &mut vcd, &mut signal_map)?;
    dbg!(&vcd.cursor);

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