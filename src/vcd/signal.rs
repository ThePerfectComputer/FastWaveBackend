// use super::utilities::{ordered_binary_lookup_u8, LookupErrors};
use super::{ScopeIdx, SignalIdx};
use num::{BigUint, Zero};

// Index to the least significant byte of a timestamp
// value on the timeline
#[derive(Debug, Copy, Clone)]
pub struct LsbIdxOfTmstmpValOnTmln(pub(super) u32);

#[derive(Debug)]
pub(super) enum SigType {
    Integer,
    Parameter,
    Real,
    Reg,
    Str,
    Wire,
    Tri1,
    Time,
}

#[derive(Debug)]
pub(super) enum TimelineQueryResults {
    BigUint(BigUint),
    String(String),
}

#[derive(Debug)]
pub(super) enum Signal {
    Data {
        name: String,
        sig_type: SigType,
        // I've seen a 0 bit signal parameter in a xilinx
        // simulation before that gets assigned 1 bit values.
        // I consider this to be bad behavior. We capture such
        // errors in the following type:
        signal_error: Option<String>,
        num_bits: Option<u16>,
        num_bytes: Option<u8>,
        // TODO : may be able to remove self_idx
        self_idx: SignalIdx,
        // A signal may take on a new value and hold that value
        // for sometime. We only need to record the value of a signal
        // when it changes(the is what VCDs tend to do).
        // A signal may need x amount of bytes to record its largest possible
        // value, so we record every single value of a given signal as a sequence
        // of x number of u8s.
        // For example, we might find that `my_signal.nums_encoded_as_fixed_width_le_u8`
        // has two 32 bit values, namely, 1 and 2, encoded as follows:
        // my_signal.nums_encoded_as_fixed_width_le_u8 = vec![1u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8];
        nums_encoded_as_fixed_width_le_u8: Vec<u8>,
        string_vals: Vec<String>,
        // we could do Vec<(LsbIdxOfTmstmpValOnTmln, u8)>, but I suspect that
        // Vec<LsbIdxOfTmstmpValOnTmln> is more cache friendly.
        // We use ``LsbIdxOfTmstmpValOnTmln`` to index into the LSB of a particular
        // timestamp encoded as the minimu length u8 sequence within
        // ``vcd.tmstmps_encoded_as_u8s``, and we use the values in
        // ``byte_len_of_num_tmstmp_vals_on_tmln`` to determine how many u8 values
        // a particular timestamp is composed of.
        lsb_indxs_of_num_tmstmp_vals_on_tmln: Vec<LsbIdxOfTmstmpValOnTmln>,
        byte_len_of_num_tmstmp_vals_on_tmln: Vec<u8>,
        lsb_indxs_of_string_tmstmp_vals_on_tmln: Vec<LsbIdxOfTmstmpValOnTmln>,
        byte_len_of_string_tmstmp_vals_on_tmln: Vec<u8>,
        scope_parent: ScopeIdx,
    },
    Alias {
        name: String,
        signal_alias: SignalIdx,
    },
}

#[derive(Debug)]
pub(super) enum LookupErrors {
    PreTimeline {
        desired_time: BigUint,
        timeline_start_time: BigUint,
    },
    EmptyTimeline,
    TimelineNotMultiple,
    OrderingFailure,
    PointsToAlias,
    NoNumBits,
    NoNumBytes,
    Other(String),
}

// these are thin type aliases primarily to make code more readable later on
type TimeStamp = BigUint;
type SignalValNum = BigUint;

impl Signal {
    pub(super) fn bytes_required(num_bits: u16, name: &String) -> Result<u8, String> {
        let bytes_required = (num_bits / 8) + if (num_bits % 8) > 0 { 1 } else { 0 };
        let bytes_required = u8::try_from(bytes_required).map_err(|_| {
            format!(
                "Error near {}:{}. Signal {name} of length num_bits requires \
                        {bytes_required} > 256 bytes.",
                file!(),
                line!()
            )
        })?;
        Ok(bytes_required)
    }
    pub fn lookup_time_and_val(
        &self,
        idx: usize,
        tmstmps_encoded_as_u8s: &Vec<u8>,
    ) -> Result<(TimeStamp, SignalValNum), LookupErrors> {
        let (
            num_bytes,
            nums_encoded_as_fixed_width_le_u8,
            lsb_indxs_of_num_tmstmp_vals_on_tmln,
            byte_len_of_num_tmstmp_vals_on_tmln,
        ) = match self {
            Signal::Data {
                num_bytes,
                nums_encoded_as_fixed_width_le_u8,
                lsb_indxs_of_num_tmstmp_vals_on_tmln,
                byte_len_of_num_tmstmp_vals_on_tmln,
                ..
            } => Ok((
                num_bytes,
                nums_encoded_as_fixed_width_le_u8,
                lsb_indxs_of_num_tmstmp_vals_on_tmln,
                byte_len_of_num_tmstmp_vals_on_tmln,
            )),
            Signal::Alias { .. } => Err(LookupErrors::PointsToAlias),
        }?;

        // get index
        let LsbIdxOfTmstmpValOnTmln(timestamp_idx) = lsb_indxs_of_num_tmstmp_vals_on_tmln[idx];
        let timestamp_idx = timestamp_idx as usize;

        // form timestamp
        let byte_len = byte_len_of_num_tmstmp_vals_on_tmln[timestamp_idx] as usize;
        let timestamp = &tmstmps_encoded_as_u8s[timestamp_idx..(timestamp_idx + byte_len)];
        let timestamp = BigUint::from_bytes_le(timestamp);

        // get signal value
        let bytes_per_value = num_bytes.ok_or_else(|| LookupErrors::NoNumBytes)?;
        let bytes_per_value = bytes_per_value as usize;
        let start_idx = idx * bytes_per_value;
        let end_idx = (idx + 1) * bytes_per_value;
        let signal_val = &nums_encoded_as_fixed_width_le_u8[start_idx..end_idx];
        let signal_val = BigUint::from_bytes_le(signal_val);

        Ok((timestamp, signal_val))
    }
    // pub(super) fn query_num_val_on_tmln(
    //     &self,
    //     //(REMOVE THIS COMMENT)below is from self
    //     nums_encoded_as_fixed_width_le_u8: &Vec<u8>,
    //     lsb_indxs_of_num_tmstmp_vals_on_tmln: &Vec<LsbIdxOfTmstmpValOnTmln>,
    //     tmstmps_encoded_as_u8s: &Vec<u8>,
    //     all_signals: &Vec<Signal>,
    //     //(REMOVE THIS COMMENT)below is from self
    //     // TODO : should this be usize?
    //     desired_time: BigUint,
    // ) -> Result<BigUint, LookupErrors> {
    //     let signal_idx = match self {
    //         Self::Data {
    //             name,
    //             sig_type,
    //             signal_error,
    //             num_bits,
    //             self_idx,
    //             ..
    //         } => {
    //             let SignalIdx(idx) = self_idx;
    //             *idx
    //         }
    //         Self::Alias { name, signal_alias } => {
    //             let SignalIdx(idx) = signal_alias;
    //             *idx
    //         }
    //     };

    //     let (
    //         nums_encoded_as_fixed_width_le_u8,
    //         lsb_indxs_of_num_tmstmp_vals_on_tmln,
    //         tmstmps_encoded_as_u8s,
    //         num_bits,
    //         name,
    //     ) = match all_signals[signal_idx] {
    //         Signal::Data {
    //             name,
    //             sig_type,
    //             signal_error,
    //             num_bits,
    //             self_idx,
    //             ref nums_encoded_as_fixed_width_le_u8,
    //             string_vals,
    //             ref lsb_indxs_of_num_tmstmp_vals_on_tmln,
    //             byte_len_of_num_tmstmp_vals_on_tmln,
    //             lsb_indxs_of_string_tmstmp_vals_on_tmln,
    //             byte_len_of_string_tmstmp_vals_on_tmln,
    //             scope_parent,
    //         } => {
    //             if num_bits.is_none() {
    //                 return Err(LookupErrors::NoNumBits);
    //             }
    //             Ok((
    //                 nums_encoded_as_fixed_width_le_u8,
    //                 lsb_indxs_of_num_tmstmp_vals_on_tmln,
    //                 tmstmps_encoded_as_u8s,
    //                 num_bits,
    //                 name,
    //             ))
    //         }
    //         Signal::Alias { name, signal_alias } => Err(LookupErrors::PointsToAlias),
    //     }?;
    //     // this signal should at least have some events, otherwise, trying to index into
    //     // an empty vector later on would fail
    //     if lsb_indxs_of_num_tmstmp_vals_on_tmln.is_empty() {
    //         return Err(LookupErrors::EmptyTimeline);
    //     }

    //     // assertion that value_sequence is a proper multiple of
    //     // timeline_markers
    //     let bytes_required =
    //         Signal::bytes_required(&num_bits, &name).map_err(|arg| LookupErrors::Other(arg))?;
    //     if lsb_indxs_of_num_tmstmp_vals_on_tmln.len()
    //         != (nums_encoded_as_fixed_width_le_u8.len() * bytes_required as usize)
    //     {
    //         return Err(LookupErrors::TimelineNotMultiple);
    //     }

    //     // let TimelineIdx(desired_time) = desired_time;

    //     // check if we're requesting a value that occurs before the recorded
    //     // start of the timeline
    //     let TimelineIdx(timeline_start_time) = timeline_cursors.first().unwrap();
    //     if desired_time < *timeline_start_time {
    //         return Err(LookupErrors::PreTimeline {
    //             desired_time: TimelineIdx(desired_time),
    //             timeline_start_time: TimelineIdx(*timeline_start_time),
    //         });
    //     }

    //     let mut lower_idx = 0usize;
    //     let mut upper_idx = timeline_cursors.len() - 1;

    //     // check if we're requesting a value that occurs beyond the end of the timeline,
    //     // if so, return the last value in this timeline
    //     let TimelineIdx(timeline_end_time) = timeline_cursors.last().unwrap();
    //     if desired_time > *timeline_end_time {
    //         let range = (value_sequence_as_bytes.len() - bytes_per_value)..;
    //         let value_by_bytes = &value_sequence_as_bytes[range];
    //         let value = BigUint::from_bytes_le(value_by_bytes);

    //         return Ok(value);
    //     }

    //     // This while loop is the meat of the lookup. Performance is log2(n),
    //     // where n is the number of events on the timeline.
    //     // We can assume that by the time we get here, that the desired_time
    //     // is an event that occurs on the timeline, given that we handle any events
    //     // occuring after or before the recorded tiimeline in the code above.
    //     while lower_idx <= upper_idx {
    //         let mid_idx = lower_idx + ((upper_idx - lower_idx) / 2);
    //         let TimelineIdx(curr_time) = timeline_cursors[mid_idx];
    //         let ordering = curr_time.cmp(&desired_time);

    //         match ordering {
    //             std::cmp::Ordering::Less => {
    //                 lower_idx = mid_idx + 1;
    //             }
    //             std::cmp::Ordering::Equal => {
    //                 let u8_timeline_start_idx = mid_idx * bytes_per_value;
    //                 let u8_timeline_end_idx = u8_timeline_start_idx + bytes_per_value;
    //                 let range = u8_timeline_start_idx..u8_timeline_end_idx;
    //                 let value_by_bytes = &value_sequence_as_bytes[range];
    //                 let value = BigUint::from_bytes_le(value_by_bytes);
    //                 return Ok(value);
    //             }
    //             std::cmp::Ordering::Greater => {
    //                 upper_idx = mid_idx - 1;
    //             }
    //         }
    //     }

    //     let idx = lower_idx - 1;
    //     let TimelineIdx(left_time) = timeline_cursors[idx];
    //     let TimelineIdx(right_time) = timeline_cursors[idx + 1];

    //     let ordered_left = left_time < desired_time;
    //     let ordered_right = desired_time < right_time;
    //     if !(ordered_left && ordered_right) {
    //         return Err(LookupErrors::OrderingFailure);
    //     }

    //     let u8_timeline_start_idx = idx * bytes_per_value;
    //     let u8_timeline_end_idx = u8_timeline_start_idx + bytes_per_value;
    //     let range = u8_timeline_start_idx..u8_timeline_end_idx;
    //     let value_by_bytes = &value_sequence_as_bytes[range];
    //     let value = BigUint::from_bytes_le(value_by_bytes);

    //     return Ok(value);
    // }
}
