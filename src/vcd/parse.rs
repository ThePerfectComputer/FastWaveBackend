// Copyright (C) 2022 Yehowshua Immanuel
// This program is distributed under both the GPLV3 license
// and the YEHOWSHUA license, both of which can be found at
// the root of the folder containing the sources for this program.
use num::BigUint;
use std::collections::HashMap;
use std::fs::File;

use super::*;

mod combinator_atoms;
use combinator_atoms::*;

mod types;
use types::*;

mod metadata;
use metadata::*;

mod scopes;
use scopes::*;

mod events;
use events::*;

pub fn parse_vcd(file: File) -> Result<VCD, String> {
    let mut word_gen = WordReader::new(file);

    let header = parse_metadata(&mut word_gen)?;

    // later, we'll need to map parsed ascii symbols to their
    // respective signal indexes
    let mut signal_map = std::collections::HashMap::new();

    // after we parse metadata, we form the VCD object
    let mut vcd = VCD {
        metadata: header,
        tmstmps_encoded_as_u8s: vec![],
        all_signals: vec![],
        all_scopes: vec![],
        root_scopes: vec![],
    };

    parse_scopes(&mut word_gen, &mut vcd, &mut signal_map)?;
    parse_events(&mut word_gen, &mut vcd, &mut signal_map)?;

    Ok(vcd)
}