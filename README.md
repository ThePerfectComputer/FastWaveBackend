Copyright - Yehowshua Immanuel

# A High performance, VCD Parser written in Rust

x/z value parsing currently fails in ``cargo test``.

A fix is coming.

## Current Features
 - pretty fast, parses 3.04 GB VCD file in ~62s on M1 Macbook Air.


# Current Limitations
Unable to handle VCD files that have signals with more than 
2^32 - 1 = 4,294,967,295 deltas/changes.

## Running

This repository comes with several smaller VCD files emitted from
various EDA tools. If you want a larger VCD file, grab one from
[here](https://drive.google.com/file/d/1pfm2qo2l8fGTHHJ8TLrg1vSGaV_TUbp2/view?usp=sharing).

The first build of the program may take some time.

``cargo run --release test-vcd-files/aldec/SPI_Write.vcd``

You can run all the tests with ``cargo test``

# TODO

## Features
 - [ ] handle signals with x or z as string
       - the one bit parsers may also need to handle string
 - move parse_orphaned_vars to scopes.rs
 - [ ] Print out git commit or release number.
 - [ ] Should be able to load waveform whilst viewing it live.
       - could be quite challenging to implement for various reasons
 - [ ] Take a look at GTKWave parser to compare efficiency.
 - [ ] re-order all signal timelines as binary balanced trees with respect to timestamps
       - support multithreaded re-ordering

## Repairs
 - [ ] make a custom date parser for possibly up to 18 different versions(that is, for each possible tool).
 - [ ] Consolidate error messages and add cursors throughout.
 - [ ] Fix warnings especially usage and restriction warnings once I'm
       able to successfully parse all sample VCDs.

## Code Consistency
 - [ ] Change error messages to line and filenames. Go through all calls to ``format!`` whilst also keeping performance in mind.

## Marketing
 - [ ] Send survey to community channel.