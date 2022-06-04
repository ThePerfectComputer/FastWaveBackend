use std::collections::BTreeMap;
use chrono::prelude::*;
use num::BigInt;

#[derive(Debug)]
struct Version(String);

#[derive(Debug)]
enum Timescale {ps, ns, us, ms, s, unit}

#[derive(Debug)]
pub(super) struct Metadata {
    date      : Option<DateTime<Utc>>,
    version   : Option<Version>,
    timescale : (Option<u32>, Timescale)}

#[derive(Debug)]
struct Scope_Idx(usize);

#[derive(Debug)]
struct Signal_Idx(usize);

#[derive(Debug)]
enum SignalGeneric{
    Signal{
        name           : String,
        timeline       : BTreeMap<BigInt, BigInt>,
        scope_parent   : Scope_Idx},
    SignalAlias{
        name          : String,
        signal_alias  : Signal_Idx}
}

#[derive(Debug)]
struct Scope {
    name          : String,
    child_signals : Vec<Signal_Idx>,
    child_scopes  : Vec<Scope_Idx>}


#[derive(Debug)]
struct VCD {
    metadata    : Metadata,
    all_signals : Vec<SignalGeneric>,
    // the root scope should always be placed at index 0
    all_scopes  : Vec<Scope>}

impl VCD {
    pub fn new() -> Self {
        let metadata = Metadata {
            date      : None,
            version   : None,
            timescale : (None, Timescale::unit)};
        VCD {
            metadata    : metadata,
            all_signals : Vec::<SignalGeneric>::new(),
            all_scopes  : Vec::<Scope>::new()}
        }
    }
