use super::*;

#[derive(Debug)]
pub(super) enum BinaryParserErrTypes {
    XValue,
    ZValue,
    UValue,
    OtherValue(char),
    TooLong,
}

// We build a quick and not so dirty bit string parser.
fn base2_str_to_byte(word: &[u8]) -> Result<u8, BinaryParserErrTypes> {
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
