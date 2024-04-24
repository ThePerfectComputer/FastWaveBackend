// Copyright (C) 2022 Yehowshua Immanuel
// This program is distributed under both the GPLV3 license
// and the YEHOWSHUA license, both of which can be found at
// the root of the folder containing the sources for this program.

/// part of the vcd parser that handles parsing the signal tree and
/// building the resulting signal tree
use std::collections::HashMap;

use super::super::reader::{curr_word, next_word, WordReader};
use super::super::signal::{SignalEnum, SignalType};
use super::super::types::{Scope, ScopeIdx, SignalIdx, VCD};

use super::combinator_atoms::{ident, tag};
use super::types::ParseResult;

pub(super) fn parse_var<R: std::io::Read>(
    word_reader: &mut WordReader<R>,
    parent_scope_idx: ScopeIdx,
    vcd: &mut VCD,
    signal_map: &mut HashMap<String, SignalIdx>,
    path: &Vec<String>,
) -> Result<(), String> {
    let (word, cursor) = next_word!(word_reader)?;
    let expected_types = [
        "event",
        "integer",
        "parameter",
        "real",
        "realtime",
        "reg",
        "string",
        "supply0",
        "supply1",
        "time",
        "tri",
        "triand",
        "trior",
        "trireg",
        "tri0",
        "tri1",
        "wand",
        "wire",
        "wor",
        "int",
        "int_s",
        "shortint",
        "int_l",
        "longint",
        "char",
        "byte",
        "logic",
        "bit",
        "shortreal",
    ];

    // $var parameter 3 a IDLE $end
    //      ^^^^^^^^^ - var_type
    let var_type = match word {
        "event" => Ok(SignalType::Event),
        "integer" => Ok(SignalType::Integer),
        "parameter" => Ok(SignalType::Parameter),
        "real" => Ok(SignalType::Real),
        "realtime" => Ok(SignalType::RealTime),
        "reg" => Ok(SignalType::Reg),
        "string" => Ok(SignalType::Str),
        "supply0" => Ok(SignalType::Supply0),
        "supply1" => Ok(SignalType::Supply1),
        "tri" => Ok(SignalType::Tri),
        "triand" => Ok(SignalType::TriAnd),
        "trior" => Ok(SignalType::TriOr),
        "trireg" => Ok(SignalType::TriReg),
        "tri0" => Ok(SignalType::Tri0),
        "tri1" => Ok(SignalType::Tri1),
        "time" => Ok(SignalType::Time),
        "wand" => Ok(SignalType::WAnd),
        "wire" => Ok(SignalType::Wire),
        "wor" => Ok(SignalType::WOr),
        "int" => Ok(SignalType::SVInt),
        "int_s" => Ok(SignalType::SVShortInt),
        "shortint" => Ok(SignalType::SVShortInt),
        "int_l" => Ok(SignalType::SVLongInt),
        "longint" => Ok(SignalType::SVLongInt),
        "logic" => Ok(SignalType::SVLogic),
        "shortreal" => Ok(SignalType::SVShortReal),
        "byte" => Ok(SignalType::SVChar),
        "char" => Ok(SignalType::SVChar),
        _ => {
            let err = format!(
                "Error near {}:{} \
                               found keyword `{word}` but expected one of \
                               {expected_types:?} on {cursor:?}",
                file!(),
                line!()
            );
            Err(err)
        }
    }?;

    let (word, cursor) = next_word!(word_reader)?;

    let parse_err = format!("failed to parse as usize on {cursor:?}");

    // $var parameter 3 a IDLE $end
    //                ^ - num_bits
    let num_bits = {
        let num_bits = word
            .parse::<usize>()
            .unwrap_or_else(|_| panic!("{}", parse_err));
        let num_bits = u32::try_from(num_bits).map_err(|_| {
            format!(
                "Error near {}:{} while parsing vcd file at {cursor:?}. \
                     This signal has {num_bits} > 2^32 - 1 bits.",
                file!(),
                line!()
            )
        })?;
        Some(num_bits)
    };

    // $var parameter 3 a IDLE $end
    //                  ^ - signal_alias
    let (word, _) = next_word!(word_reader)?;
    let signal_alias = word.to_string();

    // $var parameter 3 a IDLE $end
    //                    ^^^^ - full_signal_name(can extend until $end)
    let mut full_signal_name = Vec::<String>::new();
    let mut size = None;
    loop {
        let (word, _) = next_word!(word_reader)?;
        match word {
            "$end" => break,
            other if other.starts_with('[') => size = Some(other.to_string()),
            _ => full_signal_name.push(word.to_string()),
        }
    }
    let full_signal_name = full_signal_name.join(" ");

    let num_bytes = if num_bits.is_some() {
        let bytes_required = SignalEnum::bytes_required(num_bits.unwrap(), &full_signal_name)?;
        Some(bytes_required)
    } else {
        None
    };

    // Is the current variable an alias to a signal already encountered?
    // if so, handle ref_signal_idx accordingly, if not, add signal to hash
    // map
    let (signal, signal_idx) = match signal_map.get(&signal_alias) {
        Some(ref_signal_idx) => {
            let signal_idx = SignalIdx(vcd.all_signals.len());
            let signal = SignalEnum::Alias {
                name: full_signal_name.clone(),
                path: path
                    .iter()
                    .cloned()
                    .chain([full_signal_name])
                    .collect::<Vec<String>>(),
                signal_alias: *ref_signal_idx,
            };
            (signal, signal_idx)
        }
        None => {
            let signal_idx = SignalIdx(vcd.all_signals.len());
            signal_map.insert(signal_alias.to_string(), signal_idx);
            let signal = SignalEnum::Data {
                name: full_signal_name.clone(),
                path: path
                    .iter()
                    .cloned()
                    .chain([full_signal_name])
                    .collect::<Vec<String>>(),
                signal_type: var_type,
                index: size,
                signal_error: None,
                num_bits,
                num_bytes,
                self_idx: signal_idx,
                nums_encoded_as_fixed_width_le_u8: vec![],
                string_vals: vec![],
                lsb_indxs_of_num_tmstmp_vals_on_tmln: vec![],
                byte_len_of_num_tmstmp_vals_on_tmln: vec![],
                byte_len_of_string_tmstmp_vals_on_tmln: vec![],
                lsb_indxs_of_string_tmstmp_vals_on_tmln: vec![],
            };
            (signal, signal_idx)
        }
    };

    vcd.all_signals.push(signal);
    let ScopeIdx(parent_scope_idx_usize) = parent_scope_idx;
    let parent_scope = vcd.all_scopes.get_mut(parent_scope_idx_usize).unwrap();
    parent_scope.child_signals.push(signal_idx);

    Ok(())
}

/// Sometimes, variables can be listed outside of scopes.
/// We call these orphaned vars.
fn parse_orphaned_vars<R: std::io::Read>(
    word_reader: &mut WordReader<R>,
    vcd: &mut VCD,
    signal_map: &mut HashMap<String, SignalIdx>,
) -> Result<(), String> {
    // create scope for unscoped signals if such a scope does not
    // yet exist
    let scope_name = "Orphaned Signals";

    // set default scope_idx to the count of existing scope as we
    // generally set scope.self_idx to the number of existing scopes
    // when that particular scope was inserted
    let mut scope_idx = ScopeIdx(vcd.all_scopes.len());

    // Override scope_idx if we find a scope named "Orphaned Signals"
    // already exists
    let mut scope_already_exists = false;
    for scope in &vcd.all_scopes {
        if scope.name == scope_name {
            scope_idx = scope.self_idx;
            scope_already_exists = true;
            break;
        }
    }

    if !scope_already_exists {
        vcd.all_scopes.push(Scope {
            name: scope_name.to_string(),
            self_idx: scope_idx,
            child_signals: vec![],
            child_scopes: vec![],
        });
        vcd.root_scopes.push(scope_idx);
    }

    // we can go ahead and parse the current var as we've already encountered
    // "$var" before now.
    parse_var(word_reader, scope_idx, vcd, signal_map, &vec![])?;

    loop {
        let (word, cursor) = next_word!(word_reader)?;

        match word {
            "$var" => {
                parse_var(word_reader, scope_idx, vcd, signal_map, &vec![])?;
            }
            "$scope" => break,
            _ => {
                let msg = format!(
                    "Error near {}:{}.\
                          Expected $scope or $var, found \
                          {word} at {cursor:?}",
                    file!(),
                    line!()
                );
                Err(msg)?;
            }
        };
    }

    Ok(())
}

fn parse_scopes_inner<R: std::io::Read>(
    word_reader: &mut WordReader<R>,
    parent_scope_idx: Option<ScopeIdx>,
    vcd: &mut VCD,
    signal_map: &mut HashMap<String, SignalIdx>,
    path: &Vec<String>,
) -> Result<(), String> {
    // $scope module reg_mag_i $end
    //        ^^^^^^ - module keyword
    let (keyword, cursor) = next_word!(word_reader)?;

    let expected = ["module", "begin", "task", "function", "fork"];
    if expected.contains(&keyword) {
        Ok(())
    } else {
        let err = format!(
            "Error near {}:{}. \
                            found keyword `{keyword}` but expected one of \
                            {expected:?} on {cursor:?}",
            file!(),
            line!()
        );
        Err(err)
    }?;

    // $scope module reg_mag_i $end
    //               ^^^^^^^^^ - scope name
    let (scope_name, _) = next_word!(word_reader)?;
    // In some cases there are VCD files which have scopes without names.
    // since these occur in the wild, we'll tolerate them even if it is unclear
    // if it is supported or not by the spec.
    if scope_name != "$end" {
        let mut path = path.clone();
        path.push(scope_name.to_string());

        let curr_scope_idx = ScopeIdx(vcd.all_scopes.len());

        // register this scope as a child of the current parent scope
        // if there is a parent scope, or else we register this scope as
        // root scope
        match parent_scope_idx {
            Some(ScopeIdx(parent_scope_idx)) => {
                let parent_scope = vcd.all_scopes.get_mut(parent_scope_idx).unwrap();
                parent_scope.child_scopes.push(curr_scope_idx);
            }
            None => vcd.root_scopes.push(curr_scope_idx),
        }

        // add this scope to list of existing scopes
        vcd.all_scopes.push(Scope {
            name: scope_name.to_string(),
            self_idx: curr_scope_idx,
            child_signals: vec![],
            child_scopes: vec![],
        });

        // $scope module reg_mag_i $end
        //                         ^^^^ - end keyword
        ident(word_reader, "$end")?;

        loop {
            let (word, cursor) = next_word!(word_reader)?;
            let ParseResult { matched, residual } = tag(word, "$");
            match matched {
                // we hope that this word starts with a `$`
                "$" => {
                    match residual {
                        "scope" => {
                            // recursive - parse inside of current scope tree
                            parse_scopes_inner(
                                word_reader,
                                Some(curr_scope_idx),
                                vcd,
                                signal_map,
                                &path,
                            )?;
                        }
                        "var" => {
                            parse_var(word_reader, curr_scope_idx, vcd, signal_map, &path)?;
                        }
                        "upscope" => {
                            ident(word_reader, "$end")?;
                            break;
                        }
                        // we ignore comments
                        "comment" => loop {
                            if ident(word_reader, "$end").is_ok() {
                                break;
                            }
                        },
                        _ => {
                            let err = format!(
                                "Error near {}:{}. \
                                               found keyword `{residual}` but expected \
                                               `$scope`, `$var`, `$comment`, or `$upscope` \
                                               on {cursor:?}",
                                file!(),
                                line!()
                            );
                            return Err(err);
                        }
                    }
                }
                _ => {
                    let err = format!(
                        "Error near {}:{}. \
                                      found keyword `{matched}` but \
                                      expected `$` on {cursor:?}",
                        file!(),
                        line!()
                    );
                    return Err(err);
                }
            }
        }
    } else {
        // We'll be conservative and only allow new scopes in this case, and make the nameless
        // scope completely transparent. I.e.
        // $scope module a $end
        //   $scope module $end
        //     $scope module b $end
        //       ...
        //     $upscope
        //   $upscope
        // $upscope
        // will create `a.b`
        loop {
            let (word, cursor) = next_word!(word_reader)?;
            let ParseResult { matched, residual } = tag(word, "$");
            match matched {
                // we hope that this word starts with a `$`
                "$" => {
                    match residual {
                        "scope" => {
                            // recursive - parse inside of current scope tree
                            parse_scopes_inner(
                                word_reader,
                                parent_scope_idx,
                                vcd,
                                signal_map,
                                &path,
                            )?;
                        }
                        "upscope" => {
                            ident(word_reader, "$end")?;
                            break;
                        }
                        // we ignore comments
                        "comment" => loop {
                            if ident(word_reader, "$end").is_ok() {
                                break;
                            }
                        },
                        _ => {
                            let err = format!(
                                "Error near {}:{}. \
                                               found keyword `{residual}` in annonyoums scope but expected \
                                               `$scope`, `$comment`, or `$upscope` \
                                               on {cursor:?}",
                                file!(),
                                line!()
                            );
                            return Err(err);
                        }
                    }
                }
                _ => {
                    let err = format!(
                        "Error near {}:{}. \
                                      found keyword `{matched}` but \
                                      expected `$` on {cursor:?}",
                        file!(),
                        line!()
                    );
                    return Err(err);
                }
            }
        }
    }

    Ok(())
}

pub(super) fn parse_scopes<R: std::io::Read>(
    word_reader: &mut WordReader<R>,
    vcd: &mut VCD,
    signal_map: &mut HashMap<String, SignalIdx>,
) -> Result<(), String> {
    // get the current word
    let (word, _) = curr_word!(word_reader)?;

    // we may have orphaned vars that occur before the first scope
    if word == "$var" {
        parse_orphaned_vars(word_reader, vcd, signal_map)?;
    }

    // get the current word
    let (word, cursor) = curr_word!(word_reader)?;

    // the current word should be "scope", as `parse_orphaned_vars`(if it
    // was called), should have terminated upon encountering "$scope".
    // If `parse_orphaned_vars` was not called, `parse_scopes` should still
    // have only been called if the caller encountered the word "$scope"
    if word != "$scope" {
        let msg = format!(
            "Error near {}:{}.\
                           Expected $scope or $var, found \
                           {word} at {cursor:?}",
            file!(),
            line!()
        );
        return Err(msg);
    }

    // now for the interesting part
    parse_scopes_inner(word_reader, None, vcd, signal_map, &vec![])?;

    // let err = format!("reached end of file without parser leaving {}", function_name!());
    let expected_keywords = ["$scope", "$enddefinitions"];

    // there could be multiple signal trees, and unfortunately, we
    // can't merge the earlier call to `parse_scopes_inner` into this loop
    // because this loop gets a word from `next_word` instead of
    // `curr_word()`.
    loop {
        let (word, cursor) = next_word!(word_reader)?;

        match word {
            "$scope" => {
                parse_scopes_inner(word_reader, None, vcd, signal_map, &vec![])?;
            }
            "$enddefinitions" => {
                ident(word_reader, "$end")?;
                break;
            }
            "$comment" => {
                // although we don't store comments, we still need to advance the
                // word_reader cursor to the end of the comment
                loop {
                    if ident(word_reader, "$end").is_ok() {
                        break;
                    }
                }
            }
            _ => {
                let err = format!(
                    "Error near {}:{} \
                                found keyword `{word}` but expected one of \
                                {expected_keywords:?} on {cursor:?}",
                    file!(),
                    line!()
                );
                return Err(err);
            }
        }
    }

    Ok(())
}
