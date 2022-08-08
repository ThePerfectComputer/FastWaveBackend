use super::*;

use std::cmp::Ordering;

#[derive(Debug)]
pub(super) enum BinaryParserErrTypes {
    XValue,
    ZValue,
    UValue,
    OtherValue(char),
    TooLong,
}

// We build a quick and not so dirty bit string parser.
pub(super) fn base2_str_to_byte(word: &[u8]) -> Result<u8, BinaryParserErrTypes> {
    let mut val = 0u8;

    // shouldn't have more than 8 chars in str
    let len = word.len();
    if len > 8 {
        return Err(BinaryParserErrTypes::TooLong);
    }

    let bit_lut = [
        0b0000_0001u8,
        0b0000_0010u8,
        0b0000_0100u8,
        0b0000_1000u8,
        0b0001_0000u8,
        0b0010_0000u8,
        0b0100_0000u8,
        0b1000_0000u8,
    ];

    for (idx, chr) in word.iter().rev().enumerate() {
        match chr {
            b'1' => val = bit_lut[idx] | val,
            b'0' => {}
            b'x' | b'X' => return Err(BinaryParserErrTypes::XValue),
            b'z' | b'Z' => return Err(BinaryParserErrTypes::ZValue),
            b'u' | b'U' => return Err(BinaryParserErrTypes::UValue),
            _ => return Err(BinaryParserErrTypes::OtherValue(*chr as char)),
        }
    }

    Ok(val)
}

pub(super) fn binary_str_to_vec_u8(binary_str: &str) -> Result<Vec<u8>, BinaryParserErrTypes> {
    let mut vec_u8: Vec<u8> = Vec::new();
    let binary_str_as_bytes = binary_str.as_bytes();

    let mut tail_idx = binary_str_as_bytes.len();
    // clamp head if provided binary str is less than 8 long
    let mut head_idx = if tail_idx >= 8 {
        binary_str_as_bytes.len() - 8
    } else {
        0
    };
    while tail_idx > 0 {
        let curr_b_val = &binary_str_as_bytes[head_idx..tail_idx];
        let val_u8 = base2_str_to_byte(curr_b_val)?;
        vec_u8.push(val_u8);

        if head_idx < 8 {
            head_idx = 0
        } else {
            head_idx = head_idx - 8;
        }

        if tail_idx < 8 {
            tail_idx = 0
        } else {
            tail_idx = tail_idx - 8;
        }
    }
    Ok(vec_u8)
}

// TODO : modify ordered_binary_lookup to support VCD timeline lookup
// and return time in signature
fn compare_strs(a: &str, b: &str) -> Ordering {
    // choose the smaller of the two indices
    let upper_bound = if a.len() > b.len() { b.len() } else { a.len() };
    let a_as_bytes = a.as_bytes();
    let b_as_bytes = b.as_bytes();

    for i in 0..upper_bound {
        let a_byte = a_as_bytes[i];
        let b_byte = b_as_bytes[i];
        if a_byte > b_byte {
            return Ordering::Greater;
        }
        if b_byte > a_byte {
            return Ordering::Less;
        }
    }

    if a.len() > b.len() {
        return Ordering::Greater;
    }

    if a.len() < b.len() {
        return Ordering::Less;
    }

    return Ordering::Equal;
}

fn ordered_binary_lookup(map: &Vec<(String, SignalIdx)>, key: &str) -> Result<SignalIdx, String> {
    let mut upper_idx = map.len() - 1;
    let mut lower_idx = 0usize;

    while lower_idx <= upper_idx {
        let mid_idx = lower_idx + ((upper_idx - lower_idx) / 2);
        let (str_val, signal_idx) = map.get(mid_idx).unwrap();
        let ordering = compare_strs(key, str_val.as_str());

        match ordering {
            Ordering::Less => {
                upper_idx = mid_idx - 1;
            }
            Ordering::Equal => {
                return Ok(*signal_idx);
            }
            Ordering::Greater => {
                lower_idx = mid_idx + 1;
            }
        }
    }

    return Err(format!(
        "Error near {}:{}. Unable to find key: `{key}` in the map.",
        file!(),
        line!()
    ));
}
