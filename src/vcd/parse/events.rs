// Copyright (C) 2022 Yehowshua Immanuel
// This program is distributed under both the GPLV3 license
// and the YEHOWSHUA license, both of which can be found at
// the root of the folder containing the sources for this program.

use num::BigUint;
use std::collections::HashMap;

use super::super::reader::{next_word, Cursor, Line, Word, WordReader};
use super::super::signal::{LsbIdxOfTmstmpValOnTmln, SignalEnum};
use super::super::types::{SignalIdx, VCD};
use super::super::utilities::{binary_str_to_vec_u8, BinaryParserErrTypes};

pub(super) fn parse_events<R: std::io::Read>(
    word_reader: &mut WordReader<R>,
    vcd: &mut VCD,
    signal_map: &mut HashMap<String, SignalIdx>,
) -> Result<(), String> {
    let mut curr_tmstmp_lsb_idx = 0u32;
    let mut curr_tmstmp_len_u8 = 0u8;
    loop {
        let next_word = word_reader.next_word();

        // The following is the only case where eof is not an error.
        // If we've reached the end of the file, then there is obviously
        // nothing left to do...
        if next_word.is_none() {
            break;
        };

        let (word, cursor) = next_word.unwrap();
        let Cursor(Line(_), Word(word_in_line_idx)) = cursor;
        // we only want to match on the first word in a line
        if word_in_line_idx != 1 {
            continue;
        }
        match &word[0..1] {
            "$" => {}
            "#" => {
                let value = &word[1..];
                let (f, l) = (file!(), line!());
                let value_biguint = BigUint::parse_bytes(value.as_bytes(), 10)
                    .ok_or(())
                    .map_err(|_| {
                        format!(
                            "Error near {f}:{l}. Failed to parse {value} as BigInt at {cursor:?}"
                        )
                    })?;
                let mut value = value_biguint.to_bytes_le();
                // TODO : u32 helps with less memory, but should ideally likely be
                // configurable.
                curr_tmstmp_len_u8 = u8::try_from(value.len()).map_err(|_| {
                    format!(
                        "Error near {}:{}. Failed to convert from usize to u8.",
                        file!(),
                        line!()
                    )
                })?;
                curr_tmstmp_lsb_idx =
                    u32::try_from(vcd.tmstmps_encoded_as_u8s.len()).map_err(|_| {
                        format!(
                            "Error near {}:{}. Failed to convert from usize to u32.",
                            file!(),
                            line!()
                        )
                    })?;
                vcd.tmstmps_encoded_as_u8s.append(&mut value);
                vcd.largest_timestamp = Some(value_biguint);
            }

            // handle the case of an n bit signal whose value must be parsed
            "b" => {
                let binary_value = &word[1..];
                let observed_num_bits = u16::try_from(binary_value.len()).map_err(|_| {
                    format!(
                        "Error near {}:{}, {cursor:?}. \
                        Found signal with more than 2^16 - 1 bits.",
                        file!(),
                        line!()
                    )
                })?;

                let mut value_u8: Vec<u8> = Vec::new();
                let mut value_string = String::new();

                let mut store_as_string = false;

                // If we encounter other values than 0 or 1, we can recover from
                // the error and store the value as a string.
                // Or else, we propagate up other errors.
                match binary_str_to_vec_u8(binary_value) {
                    Ok(result) => {
                        value_u8 = result;
                    }
                    Err(
                        BinaryParserErrTypes::XValue
                        | BinaryParserErrTypes::ZValue
                        | BinaryParserErrTypes::UValue
                        | BinaryParserErrTypes::WValue
                        | BinaryParserErrTypes::HValue
                        | BinaryParserErrTypes::DashValue
                        | BinaryParserErrTypes::LValue,
                    ) => {
                        store_as_string = true;
                        // Turn to lower case for consistency
                        value_string = binary_value.to_ascii_lowercase();
                    }
                    Err(e) => {
                        let (f, l) = (file!(), line!());
                        Err(e).map_err(|e| {
                            format!("Error near {f}:{l}. Error {e:?} at {cursor:?}.")
                        })?;
                    }
                }

                // this word should be the signal alias
                let (word, cursor) = next_word!(word_reader)?;

                // lookup signal idx
                let signal_idx = signal_map.get(word).ok_or(()).map_err(|_| {
                    format!(
                        "Error near {}:{}. Failed to lookup signal {word} at {cursor:?}",
                        file!(),
                        line!()
                    )
                })?;

                let signal = vcd.dealiasing_signal_idx_to_signal_lookup_mut(signal_idx)?;

                match signal {
                    SignalEnum::Data {
                        name,
                        sig_type,
                        ref mut signal_error,
                        num_bits,
                        num_bytes,
                        nums_encoded_as_fixed_width_le_u8,
                        string_vals,
                        lsb_indxs_of_num_tmstmp_vals_on_tmln,
                        byte_len_of_num_tmstmp_vals_on_tmln,
                        lsb_indxs_of_string_tmstmp_vals_on_tmln,
                        byte_len_of_string_tmstmp_vals_on_tmln,
                        ..
                    } => {
                        // we've already identified in a prior loop iteration that the signal has
                        // an error
                        if signal_error.is_some() {
                            continue;
                        }

                        // Get the observed number of bits for the value parsed earlier
                        // and verify that it is not greater than the numbits declared
                        // when the signal was declared.
                        // Also account for the error case of a bitwidth of `None`
                        match num_bits {
                            Some(ref num_bits) => {
                                if observed_num_bits > *num_bits {
                                    let (f, l) = (file!(), line!());
                                    let msg = format!("\
                                        Error near {f}:{l}. The bitwidth for signal {name} \
                                        of sig_type {sig_type:?} is expected to be `{num_bits}` not \
                                        `{observed_num_bits}`. \
                                        This error occurred while parsing the vcd file at \
                                        {cursor:?}");
                                    *signal_error = Some(msg);
                                    continue;
                                }
                            }
                            None => {
                                let (f, l) = (file!(), line!());
                                let msg = format!(
                                    "\
                                    Error near {f}:{l}. The bitwidth for signal {name} \
                                    must be specified for a signal of type {sig_type:?}. \
                                    This error occurred while parsing the vcd file at \
                                    {cursor:?}"
                                );
                                Err(msg)?;
                            }
                        };

                        if store_as_string {
                            lsb_indxs_of_string_tmstmp_vals_on_tmln
                                .push(LsbIdxOfTmstmpValOnTmln(curr_tmstmp_lsb_idx));
                            byte_len_of_string_tmstmp_vals_on_tmln.push(curr_tmstmp_len_u8);
                            string_vals.push(value_string);
                            Ok(())
                        } else {
                            // timestamp stuff
                            lsb_indxs_of_num_tmstmp_vals_on_tmln
                                .push(LsbIdxOfTmstmpValOnTmln(curr_tmstmp_lsb_idx));
                            byte_len_of_num_tmstmp_vals_on_tmln.push(curr_tmstmp_len_u8);

                            // value stuff
                            // we may need to zero extend values
                            // so that we end up storing all values
                            // of a particular signal in a consistent
                            // amount of bytes
                            let bytes_required = num_bytes.ok_or_else(|| {
                                format!("Error near {}:{}. num_bytes empty.", file!(), line!())
                            })?;
                            let mut curr_num_bytes =
                                u8::try_from(value_u8.len()).map_err(|_| {
                                    format!(
                                        "Error near {}:{}. \
                                     Found signal {name} with with value change of greater \
                                     than 2^16 - 1 bits on {cursor:?}.",
                                        file!(),
                                        line!()
                                    )
                                })?;

                            nums_encoded_as_fixed_width_le_u8.append(&mut value_u8);
                            while curr_num_bytes < bytes_required {
                                nums_encoded_as_fixed_width_le_u8.push(0u8);
                                curr_num_bytes += 1;
                            }
                            Ok(())
                        }
                    }
                    SignalEnum::Alias { .. } => {
                        let (f, l) = (file!(), line!());
                        let msg = format!(
                            "Error near {f}:{l}, a signal alias should not point to a signal alias.\n\
                                This error occurred while parsing vcd file at {cursor:?}");
                        Err(msg)
                    }
                }?;
            }

            // handle the case of a one bit signal whose value is set to `0`
            "0" => {
                // lookup signal idx
                let hash = &word[1..];
                let signal_idx = signal_map.get(hash).ok_or(()).map_err(|_| {
                    format!(
                        "Error near {}:{}. Failed to lookup signal {hash} at {cursor:?}",
                        file!(),
                        line!()
                    )
                })?;

                let signal = vcd.dealiasing_signal_idx_to_signal_lookup_mut(signal_idx)?;

                match signal {
                    SignalEnum::Data {
                        name,
                        sig_type,
                        ref mut signal_error,
                        num_bits,
                        num_bytes,
                        nums_encoded_as_fixed_width_le_u8,
                        lsb_indxs_of_num_tmstmp_vals_on_tmln,
                        byte_len_of_num_tmstmp_vals_on_tmln,
                        ..
                    } => {
                        // if this is a bad signal, go ahead and skip it
                        if signal_error.is_some() {
                            continue;
                        }

                        // Get bitwidth and verify that it is 1.
                        // Also account for the error case of a bitwidth of `None`
                        match num_bits {
                            Some(ref num_bits) => {
                                if *num_bits != 1 {
                                    let (f, l) = (file!(), line!());
                                    let msg = format!(
                                        "\
                                        Error near {f}:{l}. The bitwidth for signal {name} \
                                        of sig_type {sig_type:?} is expected to be `1` not \
                                        `{num_bits}`. \
                                        This error occurred while parsing the vcd file at \
                                        {cursor:?}"
                                    );
                                    *signal_error = Some(msg);
                                    continue;
                                }
                            }
                            None => {
                                let (f, l) = (file!(), line!());
                                let msg = format!(
                                    "\
                                    Error near {f}:{l}. The bitwidth for signal {name} \
                                    must be specified for a signal of type {sig_type:?}. \
                                    This error occurred while parsing the vcd file at \
                                    {cursor:?}"
                                );
                                Err(msg)?;
                            }
                        };
                        // timestamp stuff
                        lsb_indxs_of_num_tmstmp_vals_on_tmln
                            .push(LsbIdxOfTmstmpValOnTmln(curr_tmstmp_lsb_idx));
                        byte_len_of_num_tmstmp_vals_on_tmln.push(curr_tmstmp_len_u8);

                        // value stuff
                        // we may need to zero extend values
                        // so that we end up storing all values
                        // of a particular signal in a consistent
                        // amount of bytes
                        let bytes_required = num_bytes.ok_or_else(|| {
                            format!("Error near {}:{}. num_bytes empty.", file!(), line!())
                        })?;
                        nums_encoded_as_fixed_width_le_u8.push(0u8);
                        let mut curr_num_bytes = 1;
                        while curr_num_bytes < bytes_required {
                            nums_encoded_as_fixed_width_le_u8.push(0u8);
                            curr_num_bytes += 1;
                        }
                        Ok(())
                    }
                    SignalEnum::Alias { .. } => {
                        let (f, l) = (file!(), line!());
                        let msg = format!(
                            "Error near {f}:{l}, a signal alias should not point to a signal alias.\n\
                             This error occurred while parsing vcd file at {cursor:?}");
                        Err(msg)
                    }
                }?;
            }

            "1" => {
                // lokup signal idx
                let hash = &word[1..];
                let signal_idx = signal_map.get(hash).ok_or(()).map_err(|_| {
                    format!(
                        "Error near {}:{}. Failed to lookup signal {hash} at {cursor:?}",
                        file!(),
                        line!()
                    )
                })?;

                let signal = vcd.dealiasing_signal_idx_to_signal_lookup_mut(signal_idx)?;

                match signal {
                    SignalEnum::Data {
                        name,
                        sig_type,
                        ref mut signal_error,
                        num_bits,
                        num_bytes,
                        nums_encoded_as_fixed_width_le_u8,
                        lsb_indxs_of_num_tmstmp_vals_on_tmln,
                        byte_len_of_num_tmstmp_vals_on_tmln,
                        ..
                    } => {
                        // if this is a bad signal, go ahead and skip it
                        if signal_error.is_some() {
                            continue;
                        }

                        // Get bitwidth and verify that it is 1.
                        // Also account for the error case of a bitwidth of `None`
                        match num_bits {
                            Some(ref num_bits) => {
                                if *num_bits != 1 {
                                    let (f, l) = (file!(), line!());
                                    let msg = format!(
                                        "\
                                        Error near {f}:{l}. The bitwidth for signal {name} \
                                        of sig_type {sig_type:?} is expected to be `1` not \
                                        `{num_bits}`. \
                                        This error occurred while parsing the vcd file at \
                                        {cursor:?}"
                                    );
                                    *signal_error = Some(msg);
                                    continue;
                                }
                            }
                            None => {
                                let (f, l) = (file!(), line!());
                                let msg = format!(
                                    "\
                                    Error near {f}:{l}. The bitwidth for signal {name} \
                                    must be specified for a signal of type {sig_type:?}. \
                                    This error occurred while parsing the vcd file at \
                                    {cursor:?}"
                                );
                                Err(msg)?;
                            }
                        };
                        // timestamp stuff
                        lsb_indxs_of_num_tmstmp_vals_on_tmln
                            .push(LsbIdxOfTmstmpValOnTmln(curr_tmstmp_lsb_idx));
                        byte_len_of_num_tmstmp_vals_on_tmln.push(curr_tmstmp_len_u8);

                        // value stuff
                        // we may need to zero extend values
                        // so that we end up storing all values
                        // of a particular signal in a consistent
                        // amount of bytes
                        let bytes_required = num_bytes.ok_or_else(|| {
                            format!("Error near {}:{}. num_bytes empty.", file!(), line!())
                        })?;
                        nums_encoded_as_fixed_width_le_u8.push(1u8);
                        let mut curr_num_bytes = 1;
                        while curr_num_bytes < bytes_required {
                            nums_encoded_as_fixed_width_le_u8.push(0u8);
                            curr_num_bytes += 1;
                        }
                        Ok(())
                    }
                    SignalEnum::Alias { .. } => {
                        let (f, l) = (file!(), line!());
                        let msg = format!(
                            "Error near {f}:{l}, a signal alias should not point to a signal alias.\n\
                             This error occurred while parsing vcd file at {cursor:?}");
                        Err(msg)
                    }
                }?;
            }

            // other one bit cases
            "x" | "X" | "z" | "Z" | "u" | "U" | "h" | "H" | "l" | "L" | "w" | "W" | "-" => {
                // Select value and turn to lowercase for consistency
                let val = word[0..1].to_ascii_lowercase();
                // lokup signal idx
                let hash = &word[1..];
                let signal_idx = signal_map.get(hash).ok_or(()).map_err(|_| {
                    format!(
                        "Error near {}:{}. Failed to lookup signal {hash} at {cursor:?}",
                        file!(),
                        line!()
                    )
                })?;

                let signal = vcd.dealiasing_signal_idx_to_signal_lookup_mut(signal_idx)?;

                match signal {
                    SignalEnum::Data {
                        name,
                        sig_type,
                        ref mut signal_error,
                        num_bits,
                        string_vals,
                        byte_len_of_string_tmstmp_vals_on_tmln,
                        lsb_indxs_of_string_tmstmp_vals_on_tmln,
                        ..
                    } => {
                        // if this is a bad signal, go ahead and skip it
                        if signal_error.is_some() {
                            continue;
                        }

                        // Get bitwidth and verify that it is 1.
                        // Also account for the error case of a bitwidth of `None`
                        match num_bits {
                            Some(ref num_bits) => {
                                if *num_bits != 1 {
                                    let (f, l) = (file!(), line!());
                                    let msg = format!(
                                        "\
                                        Error near {f}:{l}. The bitwidth for signal {name} \
                                        of sig_type {sig_type:?} is expected to be `1` not \
                                        `{num_bits}`. \
                                        This error occurred while parsing the vcd file at \
                                        {cursor:?}"
                                    );
                                    *signal_error = Some(msg);
                                    continue;
                                }
                            }
                            None => {
                                let (f, l) = (file!(), line!());
                                let msg = format!(
                                    "\
                                    Error near {f}:{l}. The bitwidth for signal {name} \
                                    must be specified for a signal of type {sig_type:?}. \
                                    This error occurred while parsing the vcd file at \
                                    {cursor:?}"
                                );
                                Err(msg)?;
                            }
                        };

                        // record timestamp at which this event occurs
                        lsb_indxs_of_string_tmstmp_vals_on_tmln
                            .push(LsbIdxOfTmstmpValOnTmln(curr_tmstmp_lsb_idx));
                        byte_len_of_string_tmstmp_vals_on_tmln.push(curr_tmstmp_len_u8);

                        // record value
                        string_vals.push(val);
                        Ok(())
                    }
                    SignalEnum::Alias { .. } => {
                        let (f, l) = (file!(), line!());
                        let msg = format!(
                            "Error near {f}:{l}, a signal alias should not point to a signal alias.\n\
                             This error occurred while parsing vcd file at {cursor:?}");
                        Err(msg)
                    }
                }?;
            }
            "s" => {
                let val = word[1..].to_string();
                let (hash, cursor) = next_word!(word_reader)?;
                // lokup signal idx
                let signal_idx = signal_map.get(hash).ok_or(()).map_err(|_| {
                    format!(
                        "Error near {}:{}. Failed to lookup signal {hash} at {cursor:?}",
                        file!(),
                        line!()
                    )
                })?;

                let signal = vcd.dealiasing_signal_idx_to_signal_lookup_mut(signal_idx)?;

                match signal {
                    SignalEnum::Data {
                        ref mut signal_error,
                        string_vals,
                        byte_len_of_string_tmstmp_vals_on_tmln,
                        lsb_indxs_of_string_tmstmp_vals_on_tmln,
                        ..
                    } => {
                        // if this is a bad signal, go ahead and skip it
                        if signal_error.is_some() {
                            continue;
                        }

                        // record timestamp at which this event occurs
                        lsb_indxs_of_string_tmstmp_vals_on_tmln
                            .push(LsbIdxOfTmstmpValOnTmln(curr_tmstmp_lsb_idx));
                        byte_len_of_string_tmstmp_vals_on_tmln.push(curr_tmstmp_len_u8);

                        // record string value
                        string_vals.push(val);
                        Ok(())
                    }
                    SignalEnum::Alias { .. } => {
                        let (f, l) = (file!(), line!());
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
