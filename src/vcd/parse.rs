// Copyright (C) 2022 Yehowshua Immanuel
// This program is distributed under both the GPLV3 license
// and the YEHOWSHUA license, both of which can be found at
// the root of the folder containing the sources for this program.

mod combinator_atoms;
mod types;
mod metadata;
mod scopes;
mod events;


pub fn parse_vcd(file: impl std::io::Read) -> Result<super::types::VCD, String> {
    let mut word_gen = super::reader::WordReader::new(file);

    let header = metadata::parse_metadata(&mut word_gen)?;

    // later, we'll need to map parsed ascii symbols to their
    // respective signal indexes
    let mut signal_map = std::collections::HashMap::new();

    // after we parse metadata, we form the VCD object
    let mut vcd = super::types::VCD {
        metadata: header,
        tmstmps_encoded_as_u8s: vec![],
        all_signals: vec![],
        all_scopes: vec![],
        root_scopes: vec![],
        largest_timestamp: None
    };

    scopes::parse_scopes(&mut word_gen, &mut vcd, &mut signal_map)?;
    events::parse_events(&mut word_gen, &mut vcd, &mut signal_map)?;

    Ok(vcd)
}
