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

impl Signal {
    pub(super) fn try_dereference_alias<'a>(
        &'a self,
        signals: &'a Vec<Signal>,
    ) -> Result<&Signal, String> {
        // dereference a signal if we need to and return a signal, else return
        // the signal itself
        let signal = match self {
            Signal::Data { .. } => self,
            Signal::Alias { name, signal_alias } => {
                let SignalIdx(idx) = signal_alias;
                &signals[*idx]
            }
        };
        match signal {
            Signal::Data { .. } => Ok(signal),
            Signal::Alias { .. } => Err(format!(
                "Error near {}:{}. A signal alias shouldn't \
                 point to a signal alias.",
                file!(),
                line!()
            )),
        }
    }
    // pub(super) fn try_dereference_alias_mut<'a>(
    //     &'a self,
    //     signals: &'a mut Vec<Signal>,
    // ) -> Result<&mut Signal, String> {
    //     // dereference a signal if we need to and return a signal, else return
    //     // the signal itself
    //     let signal = match self {
    //         Signal::Data {
    //             name,
    //             sig_type,
    //             signal_error,
    //             num_bits,
    //             self_idx,
    //             ..
    //         } => {
    //             let SignalIdx(idx) = self_idx;
    //             signals.get(*idx).unwrap()
    //         }
    //         Signal::Alias { name, signal_alias } => {
    //             let SignalIdx(idx) = signal_alias;
    //             signals.get(*idx).unwrap()
    //         }
    //     };
    //     match signal {
    //         Signal::Data { .. } => Ok(signal),
    //         Signal::Alias { .. } => Err(format!(
    //             "Error near {}:{}. A signal alias shouldn't \
    //              point to a signal alias.",
    //             file!(),
    //             line!()
    //         )),
    //     }
    // }
    pub(super) fn bytes_required(&self) -> Result<u8, String> {
        match self {
            Signal::Data {
                name,
                sig_type,
                signal_error,
                num_bits,
                ..
            } => {
                let num_bits = num_bits.ok_or_else(|| {
                    format!("Error near {}:{}. num_bits empty.", file!(), line!())
                })?;
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
            Signal::Alias { name, signal_alias } => {
                let msg = format!(
                    "Error near {}:{}. Bytes required should not be called on the signal alias {name}",
                    file!(),
                    line!()
                );
                Err(msg)
            }
        }
        // let bytes_required = (num_bits / 8) + if (num_bits % 8) > 0 { 1 } else { 0 };
    }
    // fn u8_tmstmp_to_biguint(&self, idx: usize) -> BigUint {
    //     // let lsb_idx = self.
    //     match self {
    //         Signal::Data {
    //             name,
    //             sig_type,
    //             signal_error,
    //             num_bits,
    //             self_idx,
    //             nums_encoded_as_fixed_width_le_u8,
    //             string_vals,
    //             lsb_indxs_of_num_tmstmp_vals_on_tmln,
    //             byte_len_of_num_tmstmp_vals_on_tmln,
    //             lsb_indxs_of_string_tmstmp_vals_on_tmln,
    //             byte_len_of_string_tmstmp_vals_on_tmln,
    //             scope_parent,
    //         } => {}
    //     }
    //     BigUint::zero()
    // }
    pub(super) fn query_value(&self, time: BigUint) -> Result<TimelineQueryResults, String> {
        // match
        // assert
        // ordered_binary_lookup_u8(
        //     &value_sequence_as_bytes_u8,
        //     4,
        //     &timeline_cursors,
        //     TimelineIdx(scrubbing_cursor),
        // );
        Ok(TimelineQueryResults::String("".to_string()))
    }
}
