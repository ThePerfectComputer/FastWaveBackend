use std::{fs::File};
use std::collections::HashMap;

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

fn binary_str_to_vec_u8(binary_str : &str) -> Result<Vec<u8>, String> {
    let mut vec_u8 : Vec<u8> = Vec::new();
    let mut binary_str_as_bytes = binary_str.as_bytes();

    let mut tail_idx = binary_str_as_bytes.len();
    // clamp head if provided binary str is less than 8 long
    let mut head_idx = 
        if tail_idx >= 8 
            {binary_str_as_bytes.len() - 8}
        else 
            {0};
    while {tail_idx > 0} {
        let curr_b_val = &binary_str_as_bytes[head_idx..tail_idx];
        let val_u8 = base2_str_to_byte(curr_b_val)?;
        vec_u8.push(val_u8);


        if head_idx < 8 {
            head_idx = 0
        }
        else {
            head_idx = head_idx - 8;
        }

        if tail_idx < 8 {
            tail_idx = 0
        }
        else {
            tail_idx = tail_idx - 8;
        }

    }
    Ok(vec_u8)
}

fn base2_str_to_byte(word : &[u8]) -> Result<u8, String> {
    let mut val = 0u8;

    // shouldn't have more than 8 chars in str
    let len = word.len();
    if len > 8 {
        let (f, l )= (file!(), line!());
        let err = format!(
            "Error near {f}:{l}. Base2 string has length {len} > 8.");
        return Err(err)
    }

    let bit_lut = [
        0b0000_0001u8,
        0b0000_0010u8,
        0b0000_0100u8,
        0b0000_1000u8,
        0b0001_0000u8,
        0b0010_0000u8,
        0b0100_0000u8,
        0b1000_0000u8
    ];

    for (idx, chr) in word.iter().rev().enumerate() {
        match chr {
            b'1' => {val = bit_lut[idx] | val}
            b'0' => {}
            _ => {
                let chr = *chr as char;
                let (f, l )= (file!(), line!());
                let err = format!(
                    "Error near {f}:{l}. Expected 1 or 0 in base2 string but got {chr}");
                return Err(err)
            }
        }

    }

    Ok(val)
}

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
                let mut value = binary_str_to_vec_u8(value).map_err(
                    |e| format!("Error near {f}:{l}. Failed to parse {value} as \
                    at {cursor:?} with error {e}"))?;
                    // TODO : u32 helps with less memory, but should ideally likely be
                    // configurable.
                let (f, l )= (file!(), line!());
                let start_idx = u32::try_from(vcd.timeline.len()).map_err(
                    |e| format!("Error near {f}:{l}. Failed to convert from usize to u32."))?;
                vcd.timeline_markers.push(StartIdx(start_idx));
                vcd.timeline.append(&mut value);
            }

            // handle the case of a one bit signal whose value is set to `0`
            "0" => {
                // lookup signal idx
                let hash = &word[1..];
                let (f, l )= (file!(), line!());
                let Signal_Idx(ref signal_idx) = signal_map.get(hash).ok_or(
                    format!("Error near {f}:{l}. Failed to lookup signal {hash} at {cursor:?}"))?;

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
                    self_idx, u8_timeline, u8_timeline_markers, ..} => {

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
                                    *signal_error = Some(msg);
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

                        u8_timeline_markers.push(timeline_idx);
                        u8_timeline.push(0u8);
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
                    format!("Error near {f}:{l}. Failed to lookup signal {hash} at {cursor:?}"))?;

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
                    self_idx, u8_timeline, u8_timeline_markers, scope_parent, ..} => {

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
                                    *signal_error = Some(msg);
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

                        u8_timeline_markers.push(timeline_idx);
                        u8_timeline.push(1u8);
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
            "b" => {
                let binary_value = &word[1..];
                let observed_num_bits = binary_value.len();
                let (f, l )= (file!(), line!());
                let mut value = binary_str_to_vec_u8(binary_value).map_err(
                    |e| format!("Error near {f}:{l}. Failed to parse {binary_value} as \
                    at {cursor:?} with error {e}"))?;
                
                // this word should be the signal alias
                let (word, cursor) = word_reader.next_word().unwrap();

                // lookup signal idx
                let (f, l )= (file!(), line!());
                let Signal_Idx(ref signal_idx) = signal_map.get(word).ok_or(
                    format!("Error near {f}:{l}. Failed to lookup signal {word} at {cursor:?}"))?;

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
                    self_idx, u8_timeline, u8_timeline_markers, scope_parent, ..} => {

                        if signal_error.is_some() {continue;}

                        // Get the observed number of bits for the value parsed earlier
                        // and verify that it is not greater than the numbits declared
                        // when the signal was declared.
                        // Also account for the error case of a bitwidth of `None`
                        match num_bits {
                            Some(ref num_bits) => {
                                if *num_bits > observed_num_bits {
                                    let (f, l) = (file!(), line!());
                                    let msg = format!("\
                                        Error near {f}:{l}. The bitwidth for signal {name} \
                                        of sig_type {sig_type:?} is expected to be `1` not \
                                        `{num_bits}`. \
                                        This error occurred while parsing the vcd file at \
                                        {cursor:?}");
                                        println!("Encountered bad signal {name}.");
                                    *signal_error = Some(msg);
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

                        u8_timeline_markers.push(timeline_idx);
                        u8_timeline.append(&mut value);
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
    let (word, cursor) = word_gen.curr_word().ok_or(msg)?;
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