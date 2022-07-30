//! part of the vcd parser that handles parsing the signal tree and
//! building the resulting signal tree
use function_name::named;

use super::*;

#[named]
pub(super) fn parse_var<'a>(
    word_reader      : &mut WordReader,
    parent_scope_idx : Scope_Idx,
    vcd              : &'a mut VCD,
    signal_map       : &mut HashMap<String, Signal_Idx>
) -> Result<(), String> {
    let err = format!("reached end of file without parser leaving {}", function_name!());
    let (word, cursor) = word_reader.next_word().ok_or(&err)?;
    let expected_types = ["integer", "parameter", "real", "reg", "string", "wire", "tri1", "time"];

    // $var parameter 3 a IDLE $end
    //      ^^^^^^^^^ - var_type
    let var_type = match word {
        "integer"    => {Ok(Sig_Type::Integer)}
        "parameter"  => {Ok(Sig_Type::Parameter)}
        "real"       => {Ok(Sig_Type::Real)}
        "reg"        => {Ok(Sig_Type::Reg)}
        "string"     => {Ok(Sig_Type::Str)}
        "wire"       => {Ok(Sig_Type::Wire)}
        "tri1"       => {Ok(Sig_Type::Tri1)}
        "time"       => {Ok(Sig_Type::Time)}
        _ => {
            let err = format!("found keyword `{word}` but expected one of {expected_types:?} on {cursor:?}");
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
        Sig_Type::Wire    | Sig_Type::Tri1 => {
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
    // dbg!(&signal_alias);

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
                signal_error: None,
                num_bits: no_bits,
                self_idx: signal_idx,
                timeline: vec![],
                timeline_markers: vec![],
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
pub(super) fn parse_signal_tree<'a>(
    word_reader      : &mut WordReader,
    parent_scope_idx : Option<Scope_Idx>,
    vcd              : &'a mut VCD,
    signal_map       : &mut HashMap<String, Signal_Idx>
) -> Result<(), String> {

    // $scope module reg_mag_i $end
    //        ^^^^^^ - module keyword
    let err = format!("reached end of file without parser leaving {}", function_name!());
    let (keyword, cursor) = word_reader.next_word().ok_or(&err)?;

    let expected = ["module", "begin", "task", "function"];
    if expected.contains(&keyword) {
        Ok(())
    } else {
        let err = format!("found keyword `{keyword}` but expected one of `{expected:?}` on {cursor:?}");
        Err(err)
    }?;

    // $scope module reg_mag_i $end
    //               ^^^^^^^^^ - scope name
    let (scope_name, _) = word_reader.next_word().ok_or(&err)?;

    let curr_scope_idx = Scope_Idx(vcd.all_scopes.len());
    
    // register this scope as a child of the current parent scope
    // if there is a parent scope, or else we register this scope as
    // root scope
    match parent_scope_idx {
        Some(Scope_Idx(parent_scope_idx)) => {
            let parent_scope = vcd.all_scopes.get_mut(parent_scope_idx).unwrap();
            parent_scope.child_scopes.push(curr_scope_idx);
        }
        None => {
            vcd.scope_roots.push(curr_scope_idx)
        }
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
                        parse_signal_tree(word_reader, Some(curr_scope_idx), vcd, signal_map)?;
                    }
                    "var" => {
                        parse_var(word_reader, curr_scope_idx, vcd, signal_map)?;
                    }
                    "upscope" => {
                        ident(word_reader, "$end")?;
                        break
                    }
                    // we ignore comments
                    "comment" => {
                        loop {
                            if ident(word_reader, "$end").is_ok() {break}
                        }
                    }
                    _ => {
                        let err = format!("found keyword `{residual}` but expected `$scope`, `$var`, `$comment`, or `$upscope` on {cursor:?}");
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

    Ok(())
}

#[named]
pub(super) fn parse_scopes<'a>(
    word_reader      : &mut WordReader,
    parent_scope_idx : Option<Scope_Idx>,
    vcd              : &'a mut VCD,
    signal_map       : &mut HashMap<String, Signal_Idx>
) -> Result<(), String> {
    // we've already seen `$scope`, so here we just jump right in
    parse_signal_tree(word_reader, None, vcd, signal_map)?;

    let err = format!("reached end of file without parser leaving {}", function_name!());
    let expected_keywords = ["$scope", "$enddefinitions"];

    loop {
        let (word, cursor) = word_reader.next_word().ok_or(&err)?;
        match word {
            "$scope" => {
                parse_signal_tree(word_reader, None, vcd, signal_map)?;
            }
            "$enddefinitions" => {
                ident(word_reader, "$end")?;
                break
            }
            // we ignore comments
            "comment" => {
                loop {
                    if ident(word_reader, "$end").is_ok() {break}
                }
            }
            _ => {
                let err = format!("found keyword `{word}` but expected oneof `{expected_keywords:?}` on {cursor:?}");
                return Err(err)
    
            }
        }
    }

    Ok(())
}