use chrono::prelude::*;
use itertools::Itertools;
use std::{fs::File};
use std::collections::{BTreeMap, HashMap};
use ::function_name::named;

use super::*;

mod combinator_atoms;
use combinator_atoms::*;

mod types;
use types::*;

mod metadata;
use metadata::*;

#[named]
fn parse_var<'a>(
    word_reader      : &mut WordReader,
    parent_scope_idx : Scope_Idx,
    vcd              : &'a mut VCD,
    signal_map       : &mut HashMap<String, Signal_Idx>
) -> Result<(), String> {
    let err = format!("reached end of file without parser leaving {}", function_name!());
    let (word, cursor) = word_reader.next_word().ok_or(&err)?;
    let expected_types = "[integer, parameter, real, reg, string, wire]";

    // $var parameter 3 a IDLE $end
    //      ^^^^^^^^^ - var_type
    let var_type = match word {
        "integer"    => {Ok(Sig_Type::Integer)}
        "parameter"  => {Ok(Sig_Type::Parameter)}
        "real"       => {Ok(Sig_Type::Real)}
        "reg"        => {Ok(Sig_Type::Reg)}
        "string"     => {Ok(Sig_Type::Str)}
        "wire"       => {Ok(Sig_Type::Wire)}
        _ => {
            let err = format!("found keyword `{word}` but expected one of {expected_types} on {cursor:?}");
            Err(err)
        }
    }?;

    let (word, cursor) = word_reader.next_word().ok_or(&err)?;
    let parse_err = format!("failed to parse as usize on {cursor:?}");

    // $var parameter 3 a IDLE $end
    //                ^ - no_bits
    let no_bits = match var_type {
        Sig_Type::Integer | Sig_Type::Parameter |
        Sig_Type::Real    | Sig_Type::Reg       |
        Sig_Type::Wire => {
            let no_bits = word.parse::<usize>().expect(parse_err.as_str());
            Some(no_bits)
        }
        // for strings, we don't really care what the number of bits is
        _ => {None}
    };

    // $var parameter 3 a IDLE $end
    //                  ^ - signal_alias
    let (word, cursor) = word_reader.next_word().ok_or(&err)?;
    let signal_alias = word.to_string();

    // $var parameter 3 a IDLE $end
    //                    ^^^^ - full_signal_name(can extend until $end)
    let mut full_signal_name = Vec::<String>::new();
    loop {
        let (word, cursor) = word_reader.next_word().ok_or(&err)?;
        match word {
            "$end" => {break}
            _      => {full_signal_name.push(word.to_string())}
        }
    }
    let full_signal_name = full_signal_name.join(" ");

    // Is the current variable an alias to a signal already encountered?
    // if so, handle ref_signal_idx accordingly, if not, add signal to hash
    // map
    let (signal, signal_idx) = match signal_map.get(&signal_alias) {
        Some(ref_signal_idx) => {
            let signal_idx = Signal_Idx(vcd.all_signals.len());
            let signal = Signal::Alias{
                name: full_signal_name,
                signal_alias: *ref_signal_idx};
            (signal, signal_idx)
        }
        None => {
            let signal_idx = Signal_Idx(vcd.all_signals.len());
            signal_map.insert(signal_alias.to_string(), signal_idx);
            let signal = Signal::Data{
                name: full_signal_name,
                sig_type: var_type,
                num_bits: no_bits,
                self_idx: signal_idx,
                timeline: BTreeMap::new(),
                scope_parent: parent_scope_idx };
            (signal, signal_idx)
        }
    };

    vcd.all_signals.push(signal);
    let Scope_Idx(parent_scope_idx_usize) = parent_scope_idx;
    let parent_scope = vcd.all_scopes.get_mut(parent_scope_idx_usize).unwrap();
    parent_scope.child_signals.push(signal_idx);

    Ok(())
}

#[named]
fn parse_signal_tree<'a>(
    word_reader      : &mut WordReader,
    parent_scope_idx : Option<Scope_Idx>,
    vcd              : &'a mut VCD,
    signal_map       : &mut HashMap<String, Signal_Idx>
) -> Result<(), String> {

    // $scope module reg_mag_i $end
    //        ^^^^^^ - module keyword
    let err = format!("reached end of file without parser leaving {}", function_name!());
    ident(word_reader, "module")?;

    // $scope module reg_mag_i $end
    //               ^^^^^^^^^ - scope name
    let (scope_name, _) = word_reader.next_word().ok_or(err)?;

    let curr_scope_idx = Scope_Idx(vcd.all_scopes.len());
    
    // register this scope as a child of the current parent scope
    // if there is a parent scope
    match parent_scope_idx {
        Some(Scope_Idx(parent_scope_idx)) => {
            let parent_scope = vcd.all_scopes.get_mut(parent_scope_idx).unwrap();
            parent_scope.child_scopes.push(curr_scope_idx);
        }
        None => {}
    }

    // add this scope to list of existing scopes
    vcd.all_scopes.push(
        Scope {
            name: scope_name.to_string(),
            parent_idx: parent_scope_idx,
            self_idx: curr_scope_idx,
            child_signals: vec![],
            child_scopes: vec![]
        }
    );

    // $scope module reg_mag_i $end
    //                         ^^^^ - end keyword
    ident(word_reader, "$end")?;

    let err = format!("reached end of file without parser leaving {}", function_name!());
    loop {
        let (word, cursor) = word_reader.next_word().ok_or(&err)?;
        let ParseResult{matched, residual} = tag(word, "$");
        match matched {
            // we hope that this word stars with a `$`
            "$" =>  {
                match residual {
                    "scope" => {
                        // recursive - parse inside of current scope tree
                        parse_signal_tree(word_reader, Some(curr_scope_idx), vcd, signal_map);
                    }
                    "var" => {
                        parse_var(word_reader, curr_scope_idx, vcd, signal_map)?;
                    }
                    "upscope" => {
                        ident(word_reader, "$end")?;
                        break
                    }
                    _ => {
                        let err = format!("found keyword `{residual}` but expected `$scope`, `$var`, or `$upscope` on {cursor:?}");
                        return Err(err)
                    }
                }
            }
            _ => {
                let err = format!("found keyword `{matched}` but expected `$` on {cursor:?}");
                return Err(err)
            }
        }
    }

    // TODO : remove the following Ok(()) once we add loop above
    Ok(())
}

// #[named]
// fn parse_signal_tree_outer<'a>(
//     word_reader : &mut WordReader,
//     vcd         : &'a mut VCD,
//     signal_map  : &mut HashMap<String, Signal_Idx>
// ) -> Result<(), String> {
//     // We assume we've already seen a `$scope` once by the time we reach this function,
//     // that why its call `parse_signal_tree_outer` and not just `parse_signal_tree`.
//     // If our WordReader had a `putback` function, we wouldn't need to have a 
//     // `parse_signal_tree_outer`.

//     // the current scope is the parent scope

//     // $scope module reg_mag_i $end
//     //        ^^^^^^ - module keyword
//     let err = format!("reached end of file without parser leaving {}", function_name!());
//     ident(word_reader, "module")?;

//     // $scope module reg_mag_i $end
//     //               ^^^^^^^^^ - scope name
//     let (scope_name, _) = word_reader.next_word().ok_or(err)?;

//     let curr_scope_idx = Scope_Idx(vcd.all_scopes.len());
    
//     // register this scope as a child of the current parent scope
//     let Scope_Idx(parent_scope_idx_usize) = parent_scope_idx;
//     let parent_scope = vcd.all_scopes.get_mut(parent_scope_idx_usize).unwrap();
//     parent_scope.child_scopes.push(curr_scope_idx);

//     vcd.all_scopes.push(
//         Scope {
//             name: scope_name.to_string(),
//             parent_idx: parent_scope_idx,
//             self_idx: curr_scope_idx,
//             child_signals: vec![],
//             child_scopes: vec![]
//         }
//     );

//     // $scope module reg_mag_i $end
//     //                         ^^^^ - end keyword
//     ident(word_reader, "$end")?;

//     // recursive - parse inside of current scope tree
//     parse_signal_tree(word_reader, curr_scope_idx, vcd, signal_map);

//     // ascend from parsing inside of current scope tree, expect $upscope $end
//     ident(word_reader, "$upscope")?;
//     ident(word_reader, "$end")?;

//     Ok(())
// }


pub fn parse_vcd(file : File) -> Result<(), String> {
    let mut word_gen = WordReader::new(file);

    let header = parse_metadata(&mut word_gen)?;
    dbg!(&header);

    // let (word, cursor) = word_gen.next_word().unwrap();
    // cursor.error(word).unwrap();
    let mut signal_map = std::collections::HashMap::new();

    let mut vcd = VCD{
        metadata: header,
        all_signals: vec![],
        all_scopes: vec![],
    };

    parse_signal_tree(&mut word_gen, None, &mut vcd, &mut signal_map)?;
    dbg!(&vcd.all_scopes);
    Ok(())
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