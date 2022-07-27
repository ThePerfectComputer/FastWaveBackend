use std::{fs::File};
use std::collections::HashMap;
use chrono::format::format;
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

use std::num::{IntErrorKind, ParseIntError};

use function_name::named;

/// Sometimes, variables can be listed outside of scopes.
/// We call these floating vars.
pub(super) fn parse_orphaned_vars<'a>(
    word_reader      : &mut WordReader,
    vcd              : &'a mut VCD,
    signal_map       : &mut HashMap<String, Signal_Idx>
) -> Result<(), String> {
    // create scope for unscoped signals if such a scope does not
    // yet exist
    let scope_name = "Orphaned Signals";

    // set default scope_idx to the count of existing scope as we
    // generally set scope.self_idx to the number of existing scopes
    // when that particular scope was inserted
    let mut scope_idx = Scope_Idx(vcd.all_scopes.len());

    // Override scope_idx if we find a scope named "Orphaned Signals"
    // already exists
    let mut scope_already_exists = false;
    for scope in &vcd.all_scopes {
        if scope.name == scope_name {
            scope_idx = scope.self_idx;
            scope_already_exists = true;
            break
        }
    }

    if !scope_already_exists {
        vcd.all_scopes.push(
            Scope {
                name: scope_name.to_string(),
                parent_idx: None,
                self_idx: scope_idx,
                child_signals: vec![],
                child_scopes: vec![]
            }
        );
        vcd.scope_roots.push(scope_idx);
    }
    
    // we can go ahead and parse the current var as we've already encountered
    // "$var" before now.
    parse_var(word_reader, scope_idx, vcd, signal_map)?;

    loop {
        let next_word = word_reader.next_word();

        // we shouldn't reach the end of the file here...
        if next_word.is_none() {
            let (f, l )= (file!(), line!());
            let msg = format!("Error near {f}:{l}.\
                               Reached end of file without terminating parser");
            Err(msg)?;
        };

        let (word, cursor) = next_word.unwrap();

        match word {
            "$var" => {
                parse_var(word_reader, scope_idx, vcd, signal_map)?;
            }
            "$scope" => {break}
            _ => {
                let (f, l )= (file!(), line!());
                let msg = format!("Error near {f}:{l}.\
                Expected $scope or $var, found {word} at {cursor:?}");
                Err(msg)?;
            }
        };
    }

    Ok(())
}

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
                // we try to parse the timestamp into the Value unsigned
                // variant used to hold the previous timestamp. Doing this
                // may fail with PosOverflow, which we would store in parse_ok,
                // and later try to remedy with bigger unsigned variants of Value.
                let parse_ok = 
                    if let Value::u8(_) = vcd.cursor {
                        let value = value.parse::<u8>();
                        match value {
                            Ok(value) => {
                                vcd.cursor = Value::u8(value);
                                Ok(())
                            }
                            Err(e) => Err(e)
                        }
                    }
                    else if let Value::u16(_) = vcd.cursor {
                        let value = value.parse::<u16>();
                        match value {
                            Ok(value) => {
                                vcd.cursor = Value::u16(value);
                                Ok(())
                            }
                            Err(e) => Err(e)
                        }
                    }
                    else if let Value::u32(_) = vcd.cursor {
                        let value = value.parse::<u32>();
                        match value {
                            Ok(value) => {
                                vcd.cursor = Value::u32(value);
                                Ok(())
                            }
                            Err(e) => Err(e)
                        }
                    }
                    else if let Value::u64(_) = vcd.cursor {
                        let value = value.parse::<u64>();
                        match value {
                            Ok(value) => {
                                vcd.cursor = Value::u64(value);
                                Ok(())
                            }
                            Err(e) => Err(e)
                        }
                    }
                    else {
                        let (f, l )= (file!(), line!());
                        let value = BigInt::parse_bytes(value.as_bytes(), 10).ok_or(
                            format!("Error near {f}:{l}. Failed to parse {value} as BigInt at {cursor:?}").as_str())?;
                        vcd.cursor = Value::BigInt(value);
                        Ok(())
                    };
                

                // If there was no parse error, we don't evaluate any more logic
                // in this match arm and simply continue to the next iteration of 
                // the outer loop to evaluate the next word.
                if parse_ok.is_ok() {
                    continue
                }

                // Try parsing value as u16 since there was a previous 
                // PosOverflow error, and record if this parse attempt 
                // was Ok or Err in parse_ok.
                let parse_ok = 
                    {
                        let e = parse_ok.unwrap_err();
                        // There could have been other parse errors...
                        // Return Err below if there were.
                        if e.kind() != &IntErrorKind::PosOverflow {
                            let (f, l )= (file!(), line!());
                            Err(format!("Error near {f}:{l}. {e:?}"))?;
                        }

                        match value.parse::<u16>() {
                            Ok(value) => {
                                vcd.cursor = Value::u16(value);
                                println!("switching to u16");
                                Ok(())
                            }
                            Err(e) => Err(e)
                        }
                    };

                // If there was no parse error, we don't evaluate any more logic
                // in this match arm and simply continue to the next iteration of 
                // the outer loop to evaluate the next word.
                if parse_ok.is_ok() {
                    continue
                }

                // Try parsing value as u32 since there was a previous 
                // PosOverflow error, and record if this parse attempt 
                // was Ok or Err in parse_ok.
                let parse_ok = 
                    {
                        let e = parse_ok.unwrap_err();
                        // There could have been other parse errors...
                        // Return Err below if there were.
                        if e.kind() != &IntErrorKind::PosOverflow {
                            let (f, l )= (file!(), line!());
                            Err(format!("Error near {f}:{l}. {e:?}"))?;
                        }

                        match value.parse::<u32>() {
                            Ok(value) => {
                                vcd.cursor = Value::u32(value);
                                println!("switching to u32");
                                Ok(())
                            }
                            Err(e) => Err(e)
                        }
                    };

                // If there was no parse error, we don't evaluate any more logic
                // in this match arm and simply continue to the next iteration of 
                // the outer loop to evaluate the next word.
                if parse_ok.is_ok() {
                    continue
                }

                // Try parsing value as u64 since there was a previous 
                // PosOverflow error, and record if this parse attempt 
                // was Ok or Err in parse_ok.
                let parse_ok = 
                    {
                        let e = parse_ok.unwrap_err();
                        // There could have been other parse errors...
                        // Return Err below if there were.
                        if e.kind() != &IntErrorKind::PosOverflow {
                            let (f, l )= (file!(), line!());
                            Err(format!("Error near {f}:{l}. {e:?}"))?;
                        }

                        match value.parse::<u64>() {
                            Ok(value) => {
                                vcd.cursor = Value::u64(value);
                                println!("switching to u64");
                                Ok(())
                            }
                            Err(e) => Err(e)
                        }
                    };

                // If there was no parse error, we don't evaluate any more logic
                // in this match arm and simply continue to the next iteration of 
                // the outer loop to evaluate the next word.
                if parse_ok.is_ok() {
                    continue
                }

                // Try parsing value as u64 since there was a previous 
                // PosOverflow error, and record if this parse attempt 
                // was Ok or Err in parse_ok.
                let parse_ok = 
                    {
                        let e = parse_ok.unwrap_err();
                        // There could have been other parse errors...
                        // Return Err below if there were.
                        if e.kind() != &IntErrorKind::PosOverflow {
                            let (f, l )= (file!(), line!());
                            Err(format!("Error near {f}:{l}. {e:?}"))?;
                        }

                        match value.parse::<u64>() {
                            Ok(value) => {
                                vcd.cursor = Value::u64(value);
                                println!("switching to u64");
                                Ok(())
                            }
                            Err(e) => Err(e)
                        }
                    };

                // Try parsing value as BigInt since there was a previous 
                // PosOverflow error and propagate any Result Errors.
                let e = parse_ok.unwrap_err();
                // There could have been other parse errors...
                // Return Err below if there were.
                if e.kind() != &IntErrorKind::PosOverflow {
                    let (f, l )= (file!(), line!());
                    Err(format!("Error near {f}:{l}. {e:?}"))?;
                }

                let (f, l )= (file!(), line!());
                let value = BigInt::parse_bytes(value.as_bytes(), 10).ok_or(
                    format!("Error near {f}:{l}. Failed to parse {value} as BigInt at {cursor:?}").as_str())?;
                vcd.cursor = Value::BigInt(value);
                println!("switching to BigInt");

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
                        // let pair = (0.to_bigint(), Value::u8(0));
                        let pair = (Value::u8(0), Value::u8(0));
                        let t = 0u32.to_be_bytes();
                        // timeline.push(pair);
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
            // "1" => {
            //     // lokup signal idx
            //     let hash = &word[1..].to_string();
            //     let Signal_Idx(ref signal_idx) = signal_map.get(hash).ok_or(
            //         format!("failed to lookup signal {hash} at {cursor:?}").as_str())?;

            //     // account for fact that signal idx could be an alias, so there
            //     // could be one step of indirection
            //     let signal_idx = 
            //     {
            //         let signal = vcd.all_signals.get(*signal_idx).unwrap();
            //         match signal {
            //         Signal::Data {..} => {signal_idx.clone()}
            //         Signal::Alias {name, signal_alias} => {
            //                 let Signal_Idx(ref signal_idx) = signal_alias;
            //                 signal_idx.clone()

            //             }
            //         }
            //     };

            //     // after handling potential indirection, go ahead and update the timeline
            //     // of the signal signal_idx references
            //     let signal = vcd.all_signals.get_mut(signal_idx).unwrap();
            //     match signal {
            //         Signal::Data {name, sig_type, num_bits, 
            //         self_idx, timeline, scope_parent} => {
            //             let value = 1.to_bigint().unwrap();
            //             let pair = (TimeStamp(vcd.cursor.clone()), Sig_Value::Numeric(value));
            //             timeline.push(pair);
            //             Ok(())
            //         }
            //         Signal::Alias {..} => {
            //             let (f, l )= (file!(), line!());
            //             let msg = format!(
            //                 "Error near {f}:{l}, a signal alias should not point to a signal alias.\n\
            //                     This error occurred while parsing vcd file at {cursor:?}");
            //             Err(msg)
            //         }
            //     }?;
            // }
            _ => {}
        }
    }

    Ok(())
}

pub fn parse_vcd(file : File) -> Result<VCD, String> {
    let mut word_gen = WordReader::new(file);

    let header = parse_metadata(&mut word_gen)?;

    // later, we'll need to map parsed ascii symbols to their 
    // respective signal indexes
    let mut signal_map = std::collections::HashMap::new();

    // after we parse metadata, we form VCD object
    let mut vcd = VCD{
        metadata   : header,
        cursor     : Value::u8(0),
        timeline   : vec![],
        all_signals: vec![],
        all_scopes : vec![],
        scope_roots: vec![],
    };

    // The last word parse_metadata saw determines how we proceed.
    // There may be some orphan vars we must parse first before 
    // parsing scoped vars.
    let (f, l ) = (file!(), line!());
    let msg = format!("Error near {f}:{l}. Current word empty!");
    let (word, cursor) = word_gen.curr_word().expect(msg.as_str());
    match word {
        "$scope" => {
            parse_scopes(&mut word_gen, None, &mut vcd, &mut signal_map)
        }
        "$var" => {
            parse_orphaned_vars(&mut word_gen, &mut vcd, &mut signal_map)?;
            parse_scopes(&mut word_gen, None, &mut vcd, &mut signal_map)
        }
        _ => {
            let (f, l )= (file!(), line!());
            let msg = format!("Error near {f}:{l}.\
                               Expected $scope or $var, found {word} at {cursor:?}");
            Err(msg)
        }

    }?;
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