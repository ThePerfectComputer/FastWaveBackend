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
 - [ ] support parsing dates with commas
 - [ ] Fix warning especially usage and restriction warnings once I'm
       able to successfully parse all sample VCDs.
 - [ ] Should be able to load waveform whilst viewing it live.
       - could be quite challenging to implement for various reasons

 - [ ] Consolidate error messages and add cursors throughout.
 - [ ] Consider what to do with don't care values
       will probably just convert them to strings for now.
 - [ ] Include line and possible column numbers
 - [ ] Take a look at GTKWave parser to compare effificiency.
 - [ ] Send survey to community channel.

# Probably No Longer Needed
 - [ ] Should insert nodes in BFS order