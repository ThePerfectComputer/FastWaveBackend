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
                let (f, l )= (file!(), line!());
                let value = BigInt::parse_bytes(value.as_bytes(), 10).ok_or(
                    format!("Error near {f}:{l}. Failed to parse {value} as BigInt at {cursor:?}").as_str())?;
                    // TODO : u32 helps with less memory, but should ideally likely be
                    // configurable.
                let (f, l )= (file!(), line!());
                let start_idx = u32::try_from(vcd.timeline.len()).map_err(
                    |e| format!("Error near {f}:{l}. Failed to convert from usize to u32."))?;
                vcd.timeline_markers.push(StartIdx(start_idx));
                let (_, mut value) = value.to_bytes_be();
                vcd.timeline.append(&mut value);
            }

            // handle the case of a one bit signal whose value is set to `0`
            "0" => {
                // lookup signal idx
                let hash = &word[1..];
                let (f, l )= (file!(), line!());
                let Signal_Idx(ref signal_idx) = signal_map.get(hash).ok_or(
                    format!("Error near {f}:{l}. Failed to lookup signal {hash} at {cursor:?}").as_str())?;

                // account for fact that signal idx could be an alias, so there
                // could be one step of indirection
                let signal_idx = 
                {
                    let signal = vcd.all_signals.get(*signal_idx).unwrap();
                    match signal {
                    Signal::Data {..} => {*signal_idx}
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
                    Signal::Data {name, sig_type, ref mut signal_error, num_bits, 
                    self_idx, timeline, timeline_markers, scope_parent} => {

                        // if this is a bad signal, go ahead and skip it
                        if signal_error.is_some() {continue;}

                        // Get bitwidth and verify that it is 1.
                        // Also account for the error case of a bitwidth of `None`
                        match num_bits {
                            Some(ref num_bits) => {
                                if *num_bits != 1 {
                                    let (f, l) = (file!(), line!());
                                    let msg = format!("\
                                        Error near {f}:{l}. The bitwidth for signal {name} \
                                        of sig_type {sig_type:?} is expected to be `1` not \
                                        `{num_bits}`. \
                                        This error occurred while parsing the vcd file at \
                                        {cursor:?}");
                                    signal_error.insert(msg);
                                    continue;
                                }
                            }
                            None => {
                                let (f, l) = (file!(), line!());
                                let msg = format!("\
                                    Error near {f}:{l}. The bitwidth for signal {name} \
                                    must be specified for a signal of type {sig_type:?}. \
                                    This error occurred while parsing the vcd file at \
                                    {cursor:?}");
                                Err(msg)?;
                            }
                        };

                        let (f, l )= (file!(), line!());
                        let timeline_idx = u32::try_from(vcd.timeline.len()).map_err(
                            |e| format!("Error near {f}:{l}. Failed to convert from usize to u32."))?;
                        let timeline_idx = TimelineIdx(timeline_idx);

                        let (f, l )= (file!(), line!());
                        let start_idx = u32::try_from(timeline.len()).map_err(
                            |e| format!("Error near {f}:{l}. Failed to convert from usize to u32."))?;
                        let start_idx    = StartIdx(start_idx);
                        // let pair = (timeline_idx, start_idx);
                        timeline_markers.push(timeline_idx);
                        timeline.push(0u8);
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

            // handle the case of a one bit signal whose value is set to `1`
            "1" => {
                // lokup signal idx
                let hash = &word[1..];
                let (f, l )= (file!(), line!());
                let Signal_Idx(ref signal_idx) = signal_map.get(hash).ok_or(
                    format!("Error near {f}:{l}. Failed to lookup signal {hash} at {cursor:?}").as_str())?;

                // account for fact that signal idx could be an alias, so there
                // could be one step of indirection
                let signal_idx = 
                {
                    let signal = vcd.all_signals.get(*signal_idx).unwrap();
                    match signal {
                    Signal::Data {..} => {*signal_idx}
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
                    Signal::Data {name, sig_type, ref mut signal_error, num_bits, 
                    self_idx, timeline, timeline_markers, scope_parent} => {

                        // if this is a bad signal, go ahead and skip it
                        if signal_error.is_some() {continue;}

                        // Get bitwidth and verify that it is 1.
                        // Also account for the error case of a bitwidth of `None`
                        match num_bits {
                            Some(ref num_bits) => {
                                if *num_bits != 1 {
                                    let (f, l) = (file!(), line!());
                                    let msg = format!("\
                                        Error near {f}:{l}. The bitwidth for signal {name} \
                                        of sig_type {sig_type:?} is expected to be `1` not \
                                        `{num_bits}`. \
                                        This error occurred while parsing the vcd file at \
                                        {cursor:?}");
                                    signal_error.insert(msg);
                                    continue;
                                }
                            }
                            None => {
                                let (f, l) = (file!(), line!());
                                let msg = format!("\
                                    Error near {f}:{l}. The bitwidth for signal {name} \
                                    must be specified for a signal of type {sig_type:?}. \
                                    This error occurred while parsing the vcd file at \
                                    {cursor:?}");
                                Err(msg)?;
                            }
                        };

                        let (f, l )= (file!(), line!());
                        let timeline_idx = u32::try_from(vcd.timeline.len()).map_err(
                            |e| format!("Error near {f}:{l}. Failed to convert from usize to u32."))?;
                        let timeline_idx = TimelineIdx(timeline_idx);

                        let (f, l )= (file!(), line!());
                        let start_idx = u32::try_from(timeline.len()).map_err(
                            |e| format!("Error near {f}:{l}. Failed to convert from usize to u32."))?;
                        let start_idx    = StartIdx(start_idx);
                        // let pair = (timeline_idx, start_idx);
                        timeline_markers.push(timeline_idx);
                        timeline.push(1u8);
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

            // handle the case of an n bit signal whose value must be parse
            // "b" => {
            //     // let binary_value = &word[1..];
            //     // let (f, l )= (file!(), line!());
            //     // let value = BigInt::parse_bytes(binary_value.as_bytes(), 2).ok_or(
            //     //     format!("Error near {f}:{l}. Failed to parse {binary_value} as BigInt at {cursor:?}").as_str())?;
            //     // let (_, mut value) = value.to_bytes_be();
                
            //     // this word should be the signal alias
            //     let (word, cursor) = word_reader.next_word().unwrap();

            //     // lookup signal idx
            //     let (f, l )= (file!(), line!());
            //     let Signal_Idx(ref signal_idx) = signal_map.get(word).ok_or(
            //         format!("Error near {f}:{l}. Failed to lookup signal {word} at {cursor:?}").as_str())?;

            //     // account for fact that signal idx could be an alias, so there
            //     // could be one step of indirection
            //     let signal_idx = 
            //     {
            //         let signal = vcd.all_signals.get(*signal_idx).unwrap();
            //         match signal {
            //         Signal::Data {..} => {*signal_idx}
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
            //         self_idx, timeline, timeline_markers, scope_parent} => {
            //             // get bitwidth, while accounting for the error case when
            //             // numbits is None
            //             let num_bits = {
            //                 let (f, l) = (file!(), line!());
            //                 let msg = format!("\
            //                     Error near {f}:{l}. The bitwidth for signal {name} \
            //                     must be specified for a signal of type {sig_type:?}. \
            //                     This error occurred while parsing the vcd file at \
            //                     {cursor:?}");
            //                 num_bits.as_ref().ok_or(msg)?
            //             };

            //             let (f, l )= (file!(), line!());
            //             let timeline_idx = u32::try_from(vcd.timeline.len()).map_err(
            //                 |e| format!("Error near {f}:{l}. Failed to convert from usize to u32."))?;
            //             let timeline_idx = TimelineIdx(timeline_idx);

            //             let (f, l )= (file!(), line!());
            //             let start_idx = u32::try_from(timeline.len()).map_err(
            //                 |e| format!("Error near {f}:{l}. Failed to convert from usize to u32."))?;
            //             let start_idx    = StartIdx(start_idx);
            //             let pair = (timeline_idx, start_idx);
            //             // timeline_markers.push(pair);
            //             // timeline.append(&mut [0u8, 1u8, 2u8]);
            //             timeline.push(0u8);
            //             timeline.push(1u8);
            //             timeline.push(2u8);
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
        metadata           : header,
        timeline           : vec![],
        timeline_markers   : vec![],
        all_signals        : vec![],
        all_scopes         : vec![],
        scope_roots        : vec![],
    };

    // The last word parse_metadata saw determines how we proceed.
    // There may be some orphan vars we must parse first before 
    // parsing scoped vars.
    let (f, l ) = (file!(), line!());
    let msg = format!("Error near {f}:{l}. Current word empty!");
    let (word, cursor) = word_gen.curr_word().ok_or(msg.as_str())?;
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