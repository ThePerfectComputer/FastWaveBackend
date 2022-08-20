use super::*;

pub(super) fn parse_events<'a>(
    word_reader: &mut WordReader,
    vcd: &'a mut VCD,
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
                let value = BigUint::parse_bytes(value.as_bytes(), 10)
                    .ok_or(())
                    .map_err(|_| {
                        format!(
                            "Error near {f}:{l}. Failed to parse {value} as BigInt at {cursor:?}"
                        )
                    })?;
                let mut value = value.to_bytes_le();
                // TODO : u32 helps with less memory, but should ideally likely be
                // configurable.
                curr_tmstmp_len_u8 = u8::try_from(value.len()).map_err(|_| {
                    format!(
                        "Error near {}:{}. Failed to convert from usize to u8.",
                        file!(),
                        line!()
                    )
                })?;
                vcd.tmstmps_encoded_as_u8s.append(&mut value);
                curr_tmstmp_lsb_idx =
                    u32::try_from(vcd.tmstmps_encoded_as_u8s.len()).map_err(|_| {
                        format!(
                            "Error near {}:{}. Failed to convert from usize to u8.",
                            file!(),
                            line!()
                        )
                    })?;
                // curr_tmstmp_lsb_idx = vcd.tmstmps_encoded_as_u8s.len();
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

                // If we encounter x or z in a value, we can recover from
                // the error and store the value as a string.
                // Or else, we we propagate up other errors.
                match binary_str_to_vec_u8(binary_value) {
                    Ok(result) => {
                        value_u8 = result;
                    }
                    Err(
                        BinaryParserErrTypes::XValue
                        | BinaryParserErrTypes::ZValue
                        | BinaryParserErrTypes::UValue,
                    ) => {
                        store_as_string = true;
                        value_string = binary_value.to_string();
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

                let signal = vcd.try_dereference_alias_mut(signal_idx)?;

                // we may have to dereference a signal if it's pointing at an alias
                // let signal = &vcd.all_signals[*signal_idx];
                // let signal = signal.try_dereference_alias_mut(&vcd.all_signals)?;

                match signal {
                    Signal::Data {
                        name,
                        sig_type,
                        ref mut signal_error,
                        num_bits,
                        self_idx,
                        nums_encoded_as_fixed_width_le_u8,
                        string_vals,
                        lsb_indxs_of_num_tmstmp_vals_on_tmln,
                        byte_len_of_num_tmstmp_vals_on_tmln,
                        lsb_indxs_of_string_tmstmp_vals_on_tmln,
                        byte_len_of_string_tmstmp_vals_on_tmln,
                        scope_parent,
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
                            string_vals.push(value_string);
                            Ok(())
                        } else {
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
                            lsb_indxs_of_num_tmstmp_vals_on_tmln
                                .push(LsbIdxOfTmstmpValOnTmln(curr_tmstmp_lsb_idx));
                            byte_len_of_num_tmstmp_vals_on_tmln.push(curr_num_bytes);

                            // we may need to zero extend values
                            // so that we end up storing all values
                            // of a particular signal in a consistent
                            // amount of bytes
                            let bytes_required = Signal::bytes_required(num_bits, name)?;

                            while curr_num_bytes < bytes_required {
                                // TODO: remove once library is known to be stable
                                // useful for debugging
                                // let err = format!("Error at {cursor:?}.\
                                // num_bits = {num_bits}, \
                                // observed_bits = {observed_num_bits}, \
                                // curr_num_bytes = {curr_num_bytes}, \
                                // bytes_required = {bytes_required} \
                                // for signal {name}");
                                // Err(err)?;

                                byte_len_of_num_tmstmp_vals_on_tmln.push(0u8);
                                curr_num_bytes += 1;
                            }
                            Ok(())
                        }
                    }
                    Signal::Alias { .. } => {
                        let (f, l) = (file!(), line!());
                        let msg = format!(
                            "Error near {f}:{l}, a signal alias should not point to a signal alias.\n\
                                This error occurred while parsing vcd file at {cursor:?}");
                        Err(msg)
                    }
                }?;
            }

            // handle the case of a one bit signal whose value is set to `0`
            // "0" => {
            //     // lookup signal idx
            //     let hash = &word[1..];
            //     let SignalIdx(ref signal_idx) = signal_map.get(hash).ok_or(()).map_err(|_| {
            //         format!(
            //             "Error near {}:{}. Failed to lookup signal {hash} at {cursor:?}",
            //             file!(),
            //             line!()
            //         )
            //     })?;

            //     // account for fact that signal idx could be an alias, so there
            //     // could be one step of indirection
            //     let signal_idx = {
            //         let signal = vcd.all_signals.get(*signal_idx).unwrap();
            //         match signal {
            //             Signal::Data { .. } => *signal_idx,
            //             Signal::Alias { signal_alias, .. } => {
            //                 let SignalIdx(ref signal_idx) = signal_alias;
            //                 signal_idx.clone()
            //             }
            //         }
            //     };

            //     // after handling potential indirection, go ahead and update the timeline
            //     // of the signal signal_idx references
            //     let signal = vcd.all_signals.get_mut(signal_idx).unwrap();
            //     match signal {
            //         Signal::Data {
            //             name,
            //             sig_type,
            //             ref mut signal_error,
            //             num_bits,
            //             u8_timeline,
            //             u8_timeline_markers,
            //             ..
            //         } => {
            //             // if this is a bad signal, go ahead and skip it
            //             if signal_error.is_some() {
            //                 continue;
            //             }

            //             // Get bitwidth and verify that it is 1.
            //             // Also account for the error case of a bitwidth of `None`
            //             match num_bits {
            //                 Some(ref num_bits) => {
            //                     if *num_bits != 1 {
            //                         let (f, l) = (file!(), line!());
            //                         let msg = format!(
            //                             "\
            //                             Error near {f}:{l}. The bitwidth for signal {name} \
            //                             of sig_type {sig_type:?} is expected to be `1` not \
            //                             `{num_bits}`. \
            //                             This error occurred while parsing the vcd file at \
            //                             {cursor:?}"
            //                         );
            //                         *signal_error = Some(msg);
            //                         continue;
            //                     }
            //                 }
            //                 None => {
            //                     let (f, l) = (file!(), line!());
            //                     let msg = format!(
            //                         "\
            //                         Error near {f}:{l}. The bitwidth for signal {name} \
            //                         must be specified for a signal of type {sig_type:?}. \
            //                         This error occurred while parsing the vcd file at \
            //                         {cursor:?}"
            //                     );
            //                     Err(msg)?;
            //                 }
            //             };

            //             let (f, l) = (file!(), line!());
            //             let timeline_idx = u32::try_from(vcd.tmstmps_encoded_as_u8s.len())
            //                 .map_err(|_| {
            //                     format!("Error near {f}:{l}. Failed to convert from usize to u32.")
            //                 })?;
            //             let timeline_idx = TimelineIdx(timeline_idx);

            //             u8_timeline_markers.push(timeline_idx);
            //             u8_timeline.push(0u8);
            //             Ok(())
            //         }
            //         Signal::Alias { .. } => {
            //             let (f, l) = (file!(), line!());
            //             let msg = format!(
            //                 "Error near {f}:{l}, a signal alias should not point to a signal alias.\n\
            //                  This error occurred while parsing vcd file at {cursor:?}");
            //             Err(msg)
            //         }
            //     }?;
            // }

            // "1" => {
            //     // lokup signal idx
            //     let hash = &word[1..];
            //     let SignalIdx(ref signal_idx) = signal_map.get(hash).ok_or(()).map_err(|_| {
            //         format!(
            //             "Error near {}:{}. Failed to lookup signal {hash} at {cursor:?}",
            //             file!(),
            //             line!()
            //         )
            //     })?;

            //     // account for fact that signal idx could be an alias, so there
            //     // could be one step of indirection
            //     let signal_idx = {
            //         let signal = vcd.all_signals.get(*signal_idx).unwrap();
            //         match signal {
            //             Signal::Data { .. } => *signal_idx,
            //             Signal::Alias { signal_alias, .. } => {
            //                 let SignalIdx(ref signal_idx) = signal_alias;
            //                 signal_idx.clone()
            //             }
            //         }
            //     };

            //     // after handling potential indirection, go ahead and update the timeline
            //     // of the signal signal_idx references
            //     let signal = vcd.all_signals.get_mut(signal_idx).unwrap();
            //     match signal {
            //         Signal::Data {
            //             name,
            //             sig_type,
            //             ref mut signal_error,
            //             num_bits,
            //             u8_timeline,
            //             u8_timeline_markers,
            //             ..
            //         } => {
            //             // if this is a bad signal, go ahead and skip it
            //             if signal_error.is_some() {
            //                 continue;
            //             }

            //             // Get bitwidth and verify that it is 1.
            //             // Also account for the error case of a bitwidth of `None`
            //             match num_bits {
            //                 Some(ref num_bits) => {
            //                     if *num_bits != 1 {
            //                         let (f, l) = (file!(), line!());
            //                         let msg = format!(
            //                             "\
            //                             Error near {f}:{l}. The bitwidth for signal {name} \
            //                             of sig_type {sig_type:?} is expected to be `1` not \
            //                             `{num_bits}`. \
            //                             This error occurred while parsing the vcd file at \
            //                             {cursor:?}"
            //                         );
            //                         *signal_error = Some(msg);
            //                         continue;
            //                     }
            //                 }
            //                 None => {
            //                     let (f, l) = (file!(), line!());
            //                     let msg = format!(
            //                         "\
            //                         Error near {f}:{l}. The bitwidth for signal {name} \
            //                         must be specified for a signal of type {sig_type:?}. \
            //                         This error occurred while parsing the vcd file at \
            //                         {cursor:?}"
            //                     );
            //                     Err(msg)?;
            //                 }
            //             };

            //             let (f, l) = (file!(), line!());
            //             let timeline_idx = u32::try_from(vcd.tmstmps_encoded_as_u8s.len())
            //                 .map_err(|_| {
            //                     format!("Error near {f}:{l}. Failed to convert from usize to u32.")
            //                 })?;
            //             let timeline_idx = TimelineIdx(timeline_idx);

            //             u8_timeline_markers.push(timeline_idx);
            //             u8_timeline.push(1u8);
            //             Ok(())
            //         }
            //         Signal::Alias { .. } => {
            //             let (f, l) = (file!(), line!());
            //             let msg = format!(
            //                 "Error near {f}:{l}, a signal alias should not point to a signal alias.\n\
            //                  This error occurred while parsing vcd file at {cursor:?}");
            //             Err(msg)
            //         }
            //     }?;
            // }
            // // other one bit cases
            // "x" | "X" | "z" | "Z" | "u" | "U" => {
            //     let val = word.to_string();
            //     // lokup signal idx
            //     let hash = &word[1..];
            //     let SignalIdx(ref signal_idx) = signal_map.get(hash).ok_or(()).map_err(|_| {
            //         format!(
            //             "Error near {}:{}. Failed to lookup signal {hash} at {cursor:?}",
            //             file!(),
            //             line!()
            //         )
            //     })?;

            //     // account for fact that signal idx could be an alias, so there
            //     // could be one step of indirection
            //     let signal_idx = {
            //         let signal = vcd.all_signals.get(*signal_idx).unwrap();
            //         match signal {
            //             Signal::Data { .. } => *signal_idx,
            //             Signal::Alias { signal_alias, .. } => {
            //                 let SignalIdx(ref signal_idx) = signal_alias;
            //                 signal_idx.clone()
            //             }
            //         }
            //     };

            //     // after handling potential indirection, go ahead and update the timeline
            //     // of the signal signal_idx references
            //     let signal = vcd.all_signals.get_mut(signal_idx).unwrap();
            //     match signal {
            //         Signal::Data {
            //             name,
            //             sig_type,
            //             ref mut signal_error,
            //             num_bits,
            //             string_timeline,
            //             string_timeline_markers,
            //             ..
            //         } => {
            //             // if this is a bad signal, go ahead and skip it
            //             if signal_error.is_some() {
            //                 continue;
            //             }

            //             // Get bitwidth and verify that it is 1.
            //             // Also account for the error case of a bitwidth of `None`
            //             match num_bits {
            //                 Some(ref num_bits) => {
            //                     if *num_bits != 1 {
            //                         let (f, l) = (file!(), line!());
            //                         let msg = format!(
            //                             "\
            //                             Error near {f}:{l}. The bitwidth for signal {name} \
            //                             of sig_type {sig_type:?} is expected to be `1` not \
            //                             `{num_bits}`. \
            //                             This error occurred while parsing the vcd file at \
            //                             {cursor:?}"
            //                         );
            //                         *signal_error = Some(msg);
            //                         continue;
            //                     }
            //                 }
            //                 None => {
            //                     let (f, l) = (file!(), line!());
            //                     let msg = format!(
            //                         "\
            //                         Error near {f}:{l}. The bitwidth for signal {name} \
            //                         must be specified for a signal of type {sig_type:?}. \
            //                         This error occurred while parsing the vcd file at \
            //                         {cursor:?}"
            //                     );
            //                     Err(msg)?;
            //                 }
            //             };

            //             let (f, l) = (file!(), line!());
            //             let timeline_idx = u32::try_from(vcd.tmstmps_encoded_as_u8s.len())
            //                 .map_err(|_| {
            //                     format!("Error near {f}:{l}. Failed to convert from usize to u32.")
            //                 })?;
            //             let timeline_idx = TimelineIdx(timeline_idx);

            //             string_timeline_markers.push(timeline_idx);
            //             string_timeline.push(val);
            //             Ok(())
            //         }
            //         Signal::Alias { .. } => {
            //             let (f, l) = (file!(), line!());
            //             let msg = format!(
            //                 "Error near {f}:{l}, a signal alias should not point to a signal alias.\n\
            //                  This error occurred while parsing vcd file at {cursor:?}");
            //             Err(msg)
            //         }
            //     }?;
            // }
            _ => {}
        }
    }

    Ok(())
}
