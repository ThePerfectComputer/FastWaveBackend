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
pub(super) enum Sig_Type {Integer, Parameter, Real, Reg, Str, Wire,}

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
         timeline     : BTreeMap<BigInt, Sig_Value>,
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
    pub(super) all_signals : Vec<Signal>,
    pub(super) all_scopes  : Vec<Scope>}