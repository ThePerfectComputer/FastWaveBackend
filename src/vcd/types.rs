use core::time;
use std::collections::{BTreeMap, HashMap};
use chrono::prelude::*;
use num::BigInt;

#[derive(Debug)]
pub(super) struct Version(pub String);

#[derive(Debug)]
pub(super) enum Timescale {fs, ps, ns, us, ms, s, unit}

#[derive(Debug)]
pub(super) struct Metadata {
    pub(super) date      : Option<DateTime<Utc>>,
    pub(super) version   : Option<Version>,
    pub(super) timescale : (Option<u32>, Timescale)}

#[derive(Debug, Copy, Clone)]
pub(super) struct Scope_Idx(pub(super) usize);

#[derive(Debug, Copy, Clone)]
pub(super) struct Signal_Idx(pub(super) usize);

#[derive(Debug, Copy, Clone)]
pub(super) struct TimelineIdx(pub(super) u32);

#[derive(Debug, Copy, Clone)]
pub struct StartIdx(pub(super) u32);

#[derive(Debug)]
pub(super) enum Sig_Type {Integer, Parameter, Real, Reg, Str, Wire, Tri1, Time}

#[derive(Debug)]
pub(super) enum TimeStamp {
    u8(u8), 
    u16(u16),
    u32(u32),
    u64(u64),
    BigInt(BigInt),
}

#[derive(Debug, Clone)]
pub(super) enum Value {
    u8(u8), 
    u16(u16),
    u32(u32),
    u64(u64),
    BigInt(BigInt),
}

pub type BigNum = Vec<u8>;

#[derive(Debug)]
pub(super) enum Sig_Value {
    Numeric(u64),
    NonNumeric(String)}

#[derive(Debug)]
pub(super) enum Signal{
    Data{
         name             : String,
         sig_type         : Sig_Type,
         // I've seen a 0 bit signal parameter in a xilinx
         // simulation before that gets assigne 1 bit values.
         // I consider this to be bad behavior. We capture such
         // errors in the following type.
         signal_error     : Option<String>,
         num_bits         : Option<usize>,
         // TODO : may be able to remove self_idx
         self_idx         : Signal_Idx,
         timeline         : Vec<u8>,
         timeline_markers : Vec<(TimelineIdx)>,
         scope_parent : Scope_Idx},
    Alias{
         name         : String,
         signal_alias : Signal_Idx}
}

#[derive(Debug)]
pub(super) struct Scope {
    pub(super) name          : String,

    pub(super) parent_idx    : Option<Scope_Idx>,
    // TODO : may be able to remove self_idx
    pub(super) self_idx      : Scope_Idx,

    pub(super) child_signals : Vec<Signal_Idx>,
    pub(super) child_scopes  : Vec<Scope_Idx>}


// TODO: document how timeline is represented
#[derive(Debug)]
pub struct VCD {
    pub(super) metadata    : Metadata,
    pub timeline           : Vec<u8>,
    pub timeline_markers   : Vec<StartIdx>,
    pub(super) all_signals : Vec<Signal>,
    pub(super) all_scopes  : Vec<Scope>,
    pub(super) scope_roots : Vec<Scope_Idx>}

impl VCD {
    // TODO : make this a generic traversal function that applies specified 
    // functions upon encountering scopes and signals
    fn print_scope_tree(
        &self,
        root_scope_idx : Scope_Idx,
        depth : usize)
    {
        let all_scopes  = &self.all_scopes;
        let all_signals = &self.all_signals;

        let indent = " ".repeat(depth * 4);
        let Scope_Idx(root_scope_idx) = root_scope_idx;
        let root_scope = &all_scopes[root_scope_idx];
        let root_scope_name = &root_scope.name;

        println!("{indent}scope: {root_scope_name}");

        for Signal_Idx(ref signal_idx) in &root_scope.child_signals {
            let child_signal = &all_signals[*signal_idx];
            let name = match child_signal {
                Signal::Data{name, ..} => {name}
                Signal::Alias{name, ..} => {name}
            };
            println!("{indent} - sig: {name}")
        }
        println!();

        for scope_idx in &root_scope.child_scopes {
            // let Scope_Idx(ref scope_idx_usize) = scope_idx;
            // let child_scope = &all_scopes[*scope_idx_usize];
            self.print_scope_tree(*scope_idx, depth+1);
        }
        // let root = vcd.all_scopes;
    }

    pub fn print_scopes(&self) {
        for scope_root in &self.scope_roots {
            self.print_scope_tree(*scope_root, 0);
        }
    }

    // pub fn average_len(&self) -> f64{
    //     let mut total_lens = 0.0;
    //     for el in &self.timeline {
    //         total_lens += el.len() as f64;
    //     }

    //     return total_lens/(self.timeline.len() as f64);
    // }

    // pub fn total_len(&self) -> usize{
    //     let mut total_lens = 0usize;
    //     for el in &self.timeline {
    //         total_lens += el.len();
    //     }

    //     return total_lens;
    // }

    pub fn print_longest_signal(&self) {
        let mut idx = 0usize;
        let mut max_len = 0usize;
        let mut signal_name = String::new();

        for signal in &self.all_signals {
            match signal {
                Signal::Alias {..} => {}
                Signal::Data { 
                    name, 
                    sig_type, 
                    num_bits, 
                    self_idx, 
                    timeline, 
                    .. } => {
                        if timeline.len() > max_len {
                            max_len = timeline.len();
                            let Signal_Idx(idx_usize) = self_idx;
                            idx = *idx_usize;
                            signal_name = name.clone();
                        }

                    }
            }
        }

        dbg!((idx, max_len, signal_name));
    }
}