use super::Signal;
use chrono::prelude::*;

#[derive(Debug)]
pub(super) struct Version(pub String);

#[derive(Debug)]
pub(super) enum Timescale {
    Fs,
    Ps,
    Ns,
    Us,
    Ms,
    S,
    Unit,
}

#[derive(Debug)]
pub(super) struct Metadata {
    pub(super) date: Option<DateTime<Utc>>,
    pub(super) version: Option<Version>,
    pub(super) timescale: (Option<u32>, Timescale),
}

// We do a lot of arena allocation in this codebase.
#[derive(Debug, Copy, Clone)]
pub(super) struct ScopeIdx(pub(super) usize);

#[derive(Debug, Copy, Clone, PartialEq)]
pub(super) struct SignalIdx(pub(super) usize);

#[derive(Debug)]
pub(super) struct Scope {
    pub(super) name: String,

    pub(super) parent_idx: Option<ScopeIdx>,
    pub(super) self_idx: ScopeIdx,

    pub(super) child_signals: Vec<SignalIdx>,
    pub(super) child_scopes: Vec<ScopeIdx>,
}

#[derive(Debug)]
pub struct VCD {
    pub(super) metadata: Metadata,
    // Since we only need to store values when there is an actual change
    // in the timeline, we keep a vector that stores the time at which an
    // event occurs. Time t is always stored/encoded as the minimum length sequence
    // of u8.
    // We essentially fill ``tmstmps_encoded_as_u8s`` with BigUints converted
    // to sequences of little endian u8s.
    // It is up to the signals to keep track of the start/stop indices in the
    // vector of u8s that constitute a timestamp value. Signals don't have to
    // keep track of all timestamp values, a given signal only needs to keep
    // track of the timestamps at which the given signal value changes.
    pub tmstmps_encoded_as_u8s: Vec<u8>,
    pub(super) all_signals: Vec<Signal>,
    pub(super) all_scopes: Vec<Scope>,
    pub(super) root_scopes: Vec<ScopeIdx>,
}

impl VCD {
    /// We take in a Signal and attempt to dereference that signal if it is of
    /// variant ``Signal::Alias``. If it is of variant ``Signal::Alias`` and points to
    /// another alias, that's an error. Otherwise, we return the ``Signal::Data``
    /// pointed to by the ``Signal::Alias``.
    /// If the Signal is of varint ``Signal::Data``, then that can be returned directly.
    pub(super) fn try_dereference_alias_mut<'a>(
        &'a mut self,
        idx: &SignalIdx,
    ) -> Result<&'a mut Signal, String> {
        // get the signal pointed to be SignalIdx from the arena
        let SignalIdx(idx) = idx;
        let signal = &self.all_signals[*idx];

        // dereference signal if Signal::Alias, or keep idx if Signal::Data
        let signal_idx = match signal {
            Signal::Data {
                name,
                sig_type,
                signal_error,
                num_bits,
                self_idx,
                ..
            } => *self_idx,
            Signal::Alias { name, signal_alias } => *signal_alias,
        };

        // Should now  point to Signal::Data variant, or else there's an error
        let SignalIdx(idx) = signal_idx;
        let signal = self.all_signals.get_mut(idx).unwrap();
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
    pub(super) fn try_dereference_alias<'a>(
        &'a self,
        idx: &SignalIdx,
    ) -> Result<&'a Signal, String> {
        // get the signal pointed to be SignalIdx from the arena
        let SignalIdx(idx) = idx;
        let signal = &self.all_signals[*idx];

        // dereference signal if Signal::Alias, or keep idx if Signal::Data
        let signal_idx = match signal {
            Signal::Data {
                name,
                sig_type,
                signal_error,
                num_bits,
                self_idx,
                ..
            } => *self_idx,
            Signal::Alias { name, signal_alias } => *signal_alias,
        };

        // Should now  point to Signal::Data variant, or else there's an error
        let SignalIdx(idx) = signal_idx;
        let signal = self.all_signals.get(idx).unwrap();
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
}
