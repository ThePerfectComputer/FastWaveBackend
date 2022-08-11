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

#[derive(Debug, Copy, Clone)]
pub(super) struct ScopeIdx(pub(super) usize);

#[derive(Debug, Copy, Clone, PartialEq)]
pub(super) struct SignalIdx(pub(super) usize);

#[derive(Debug, Copy, Clone)]
pub(super) struct TimelineIdx(pub(super) u32);

#[derive(Debug, Copy, Clone)]
pub struct StartIdx(pub(super) u32);

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
pub(super) enum Signal {
    Data {
        name: String,
        sig_type: SigType,
        // I've seen a 0 bit signal parameter in a xilinx
        // simulation before that gets assigned 1 bit values.
        // I consider this to be bad behavior. We capture such
        // errors in the following type.
        signal_error: Option<String>,
        num_bits: Option<usize>,
        // TODO : may be able to remove self_idx
        self_idx: SignalIdx,
        // we could encounter a mix of pure values and strings
        // for the same signal timeline
        u8_timeline: Vec<u8>,
        u8_timeline_markers: Vec<TimelineIdx>,
        string_timeline: Vec<String>,
        string_timeline_markers: Vec<TimelineIdx>,
        scope_parent: ScopeIdx,
    },
    Alias {
        name: String,
        signal_alias: SignalIdx,
    },
}

#[derive(Debug)]
pub(super) struct Scope {
    pub(super) name: String,

    pub(super) parent_idx: Option<ScopeIdx>,
    pub(super) self_idx: ScopeIdx,

    pub(super) child_signals: Vec<SignalIdx>,
    pub(super) child_scopes: Vec<ScopeIdx>,
}

// TODO: document how timeline is represented
#[derive(Debug)]
pub struct VCD {
    pub(super) metadata: Metadata,
    // since we only need to store values when there is an actual change
    // in the timeline, we keep a vector that stores the time at which an
    // event occurs. Time t is always stored as the minimum length sequence
    // of u8.
    pub timeline: Vec<u8>,
    // we need to keep track of where a given time t sequence of u8 begins
    // and ends in the timeline vector.
    pub timeline_markers: Vec<StartIdx>,
    pub(super) all_signals: Vec<Signal>,
    pub(super) all_scopes: Vec<Scope>,
    pub(super) scope_roots: Vec<ScopeIdx>,
}

impl VCD {
    // TODO : make this a generic traversal function that applies specified
    // functions upon encountering scopes and signals
    fn print_scope_tree(&self, root_scope_idx: ScopeIdx, depth: usize) {
        let all_scopes = &self.all_scopes;
        let all_signals = &self.all_signals;

        let indent = " ".repeat(depth * 4);
        let ScopeIdx(root_scope_idx) = root_scope_idx;
        let root_scope = &all_scopes[root_scope_idx];
        let root_scope_name = &root_scope.name;

        println!("{indent}scope: {root_scope_name}");

        for SignalIdx(ref signal_idx) in &root_scope.child_signals {
            let child_signal = &all_signals[*signal_idx];
            let name = match child_signal {
                Signal::Data { name, .. } => name,
                Signal::Alias { name, .. } => name,
            };
            println!("{indent} - sig: {name}")
        }
        println!();

        for scope_idx in &root_scope.child_scopes {
            self.print_scope_tree(*scope_idx, depth + 1);
        }
    }

    pub fn print_scopes(&self) {
        for scope_root in &self.scope_roots {
            self.print_scope_tree(*scope_root, 0);
        }
    }

    pub fn print_longest_signal(&self) {
        let mut idx = 0usize;
        let mut max_len = 0usize;
        let mut signal_name = String::new();

        for signal in &self.all_signals {
            match signal {
                Signal::Alias { .. } => {}
                Signal::Data {
                    name,
                    self_idx,
                    u8_timeline,
                    ..
                } => {
                    if u8_timeline.len() > max_len {
                        max_len = u8_timeline.len();
                        let SignalIdx(idx_usize) = self_idx;
                        idx = *idx_usize;
                        signal_name = name.clone();
                    }
                }
            }
        }

        dbg!((idx, max_len, signal_name));
    }
}
