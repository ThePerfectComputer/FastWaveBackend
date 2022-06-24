# Disclaimer
PROPRIETARY - Copyright - Yehowshua Immanuel

# The Beginnings of a high-performance, low memory footprint VCD Viewer in Rust for massive multi-GB waveforms

## Current Features
 - very fast
 - loads 400MB of VCD waveform per second on an 8 core 2017 desktop CPU with NVMe storage
 - consumes roughly between 10 - 50MB of memory per GB of waveform

## Planed Features
 - elegant/pretty UI
 - can be easily ported to work in browser via webassembly
 - allows high-performance custom Rust plugins to manipulate and
   generate new waveforms live

## Running

Make sure you have a test vcd file to get you started. You can grab
a large VCD file from
[here](https://drive.google.com/file/d/1pfm2qo2l8fGTHHJ8TLrg1vSGaV_TUbp2/view?usp=sharing).

The first build of the program may take some time.

``cargo run --release test-vcd-files/aldec/SPI_Write.vcd``

You can run all the tests with ``cargo test``

# TODO
 - [x] We need a way to merge lines.
 - [x] We need to start regression testing the parser over all files
 - [x] Decide if I want to return option types
 - [x] Propagate all to question mark unwrap types.
 - [x] Don't want variation in hh:mm:ss
 - [x] parser_atoms -> combinator_atoms
 - [x] make parse/types.rs
 - [x] remove/replace calls to match_not_empty
 - [x] Split ``parse.rs``. It's getting too large.
 - [x] move list of files to separate test file/folder
 - [ ] support parsing dates with commas
 - [ ] Fix warning especially usage and restriction warnings once I'm
       able to successfully parse all sample VCDs.

 - [ ] Consolidate error messages and add cursors.
 - [ ] Consider what to do with don't care values
      will probably just convert them to strings for now.
 - [ ] Include line and possible column numbers
 - [ ] Change states to lowercase
 - [ ] Take a look at GTKWave parser to compare effificiency.
 - [ ] Send survey to community channel.

# Probably No Longer Needed
 - [ ] Should insert nodes in BFS order