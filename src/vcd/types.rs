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

#[derive(Debug)]
pub(super) enum Sig_Type {Integer, Parameter, Real, Reg, Str, Wire, Tri1, Time}

#[derive(Debug)]
pub(super) struct TimeStamp(BigInt);

#[derive(Debug)]
pub(super) enum Sig_Value {
    Numeric(BigInt),
    NonNumeric(String)}

#[derive(Debug)]
pub(super) enum Signal{
    Data{
         name         : String,
         sig_type     : Sig_Type,
         num_bits     : Option<usize>,
         // TODO : may be able to remove self_idx
         self_idx     : Signal_Idx,
         timeline      : Vec<(TimeStamp, Sig_Value)>,
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


#[derive(Debug)]
pub struct VCD {
    pub(super) metadata    : Metadata,
    pub(super) cursor      : BigInt,
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
}