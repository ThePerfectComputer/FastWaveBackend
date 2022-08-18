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

use num::{BigUint, Zero};

#[derive(Debug)]
pub(super) enum LookupErrors {
    PreTimeline {
        desired_time: TimelineIdx,
        timeline_start_time: TimelineIdx,
    },
    EmptyTimeline,
    TimelineNotMultiple,
    OrderingFailure,
}

pub(super) fn ordered_binary_lookup_u8(
    value_sequence_as_bytes: &Vec<u8>,
    bytes_per_value: usize,
    timeline_cursors: &Vec<TimelineIdx>,
    desired_time: TimelineIdx,
) -> Result<BigUint, LookupErrors> {
    // timeline must not be empty
    if timeline_cursors.is_empty() {
        return Err(LookupErrors::EmptyTimeline);
    }

    // assertion that value_sequence is a proper multiple of
    // timeline_markers
    if value_sequence_as_bytes.len() != (timeline_cursors.len() * bytes_per_value) {
        return Err(LookupErrors::TimelineNotMultiple);
    }

    let TimelineIdx(desired_time) = desired_time;

    // check if we're requesting a value that occurs before the recorded
    // start of the timeline
    let TimelineIdx(timeline_start_time) = timeline_cursors.first().unwrap();
    if desired_time < *timeline_start_time {
        return Err(LookupErrors::PreTimeline {
            desired_time: TimelineIdx(desired_time),
            timeline_start_time: TimelineIdx(*timeline_start_time),
        });
    }

    let mut lower_idx = 0usize;
    let mut upper_idx = timeline_cursors.len() - 1;

    // check if we're requesting a value that occurs beyond the end of the timeline,
    // if so, return the last value in this timeline
    let TimelineIdx(timeline_end_time) = timeline_cursors.last().unwrap();
    if desired_time > *timeline_end_time {
        let range = (value_sequence_as_bytes.len() - bytes_per_value)..;
        let value_by_bytes = &value_sequence_as_bytes[range];
        let value = BigUint::from_bytes_le(value_by_bytes);

        return Ok(value);
    }

    // This while loop is the meat of the lookup. Performance is log2(n),
    // where n is the number of events on the timeline.
    // We can assume that by the time we get here, that the desired_time
    // is an event that occurs on the timeline, given that we handle any events
    // occuring after or before the recorded tiimeline in the code above.
    while lower_idx <= upper_idx {
        let mid_idx = lower_idx + ((upper_idx - lower_idx) / 2);
        let TimelineIdx(curr_time) = timeline_cursors[mid_idx];
        let ordering = curr_time.cmp(&desired_time);

        match ordering {
            std::cmp::Ordering::Less => {
                lower_idx = mid_idx + 1;
            }
            std::cmp::Ordering::Equal => {
                let u8_timeline_start_idx = mid_idx * bytes_per_value;
                let u8_timeline_end_idx = u8_timeline_start_idx + bytes_per_value;
                let range = u8_timeline_start_idx..u8_timeline_end_idx;
                let value_by_bytes = &value_sequence_as_bytes[range];
                let value = BigUint::from_bytes_le(value_by_bytes);
                return Ok(value);
            }
            std::cmp::Ordering::Greater => {
                upper_idx = mid_idx - 1;
            }
        }
    }

    let idx = lower_idx - 1;
    let TimelineIdx(left_time) = timeline_cursors[idx];
    let TimelineIdx(right_time) = timeline_cursors[idx + 1];

    let ordered_left = left_time < desired_time;
    let ordered_right = desired_time < right_time;
    if !(ordered_left && ordered_right) {
        return Err(LookupErrors::OrderingFailure);
    }

    let u8_timeline_start_idx = idx * bytes_per_value;
    let u8_timeline_end_idx = u8_timeline_start_idx + bytes_per_value;
    let range = u8_timeline_start_idx..u8_timeline_end_idx;
    let value_by_bytes = &value_sequence_as_bytes[range];
    let value = BigUint::from_bytes_le(value_by_bytes);

    return Ok(value);
}
