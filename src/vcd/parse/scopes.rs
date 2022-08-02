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
        Sig_Type::Wire    | Sig_Type::Tri1      |
        Sig_Type::Time => {
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
                u8_timeline: vec![],
                u8_timeline_markers: vec![],
                string_timeline: vec![],
                string_timeline_markers: vec![],
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

/// Sometimes, variables can be listed outside of scopes.
/// We call these orphaned vars.
fn parse_orphaned_vars<'a>(
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
            // we hope that this word starts with a `$`
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
    // get the current word
    let (f, l ) = (file!(), line!());
    let msg = format!("Error near {f}:{l}. Current word empty!");
    let (word, cursor) = word_reader.curr_word().ok_or(msg)?;

    // we may have orphaned vars that occur before the first scope
    if word == "$var" {
        parse_orphaned_vars(word_reader, vcd, signal_map)?;
    } 
    
    // get the current word
    let (f, l ) = (file!(), line!());
    let msg = format!("Error near {f}:{l}. Current word empty!");
    let (word, cursor) = word_reader.curr_word().ok_or(msg)?;

    // the current word should be "scope", as `parse_orphaned_vars`(if it
    // was called), should have terminated upon encountering "$scope".
    // If `parse_orphaned_vars` was not called, `parse_scopes` should still
    // have only been called if the caller encountered the word "$scope"
    if word != "$scope" {
        let (f, l )= (file!(), line!());
        let msg = format!("Error near {f}:{l}.\
                            Expected $scope or $var, found {word} at {cursor:?}");
        return Err(msg)
    }

    // now for the interesting part
    parse_signal_tree(word_reader, None, vcd, signal_map)?;

    let err = format!("reached end of file without parser leaving {}", function_name!());
    let expected_keywords = ["$scope", "$enddefinitions"];

    // there could be multiple signal trees, and unfortunately, we
    // can't merge the earlier call to `parse_signal_tree` into this loop
    // because this loop gets a word from `next_word` instead of 
    // `curr_word()`.
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
            "comment" => {
                // although we don't store comments, we still need to advance the
                // word_reader cursor to the end of the comment
                loop {
                    if ident(word_reader, "$end").is_ok() {break}
                }
            }
            _ => {
                let err = format!("found keyword `{word}` but expected one \
                of `{expected_keywords:?}` on {cursor:?}");
                return Err(err)
    
            }
        }
    }

    Ok(())
}