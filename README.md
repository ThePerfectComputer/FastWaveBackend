# The Beginnings of a high-performance, low memory footprint VCD Viewer in Rust for massive multi-GB waveforms

## Features
 - very fast
 - loads 200MB of VCD waveform per second on an 8 core 2017 desktop CPU with NVMe storage
 - consumes roughly between 10 - 50MB of memory per GB of waveform
 - elegant/pretty UI
 - can be easily ported to work in browser via webassembly
 - allows high-performance custom Rust plugins to manipulate and
   generate new waveforms live

## Running

Make sure you have a test vcd file to get you started. You can grab
a large VCD file from
[here](https://drive.google.com/file/d/1pfm2qo2l8fGTHHJ8TLrg1vSGaV_TUbp2/view?usp=sharing).

The first build of the program may take some time.

``cargo  run --release -- path/to/vcd/file``

## TODO
 - [x] Test positions with seeking
 - [x] vcd should be argument
 - [x] structure to store stream position against timestamp as string
 - [x] structure to store stream position against timestamp as BigInt

### April 14
 - [ ] store timestamps to struct
 - [ ] Get file loading status
 - [ ] Get all signal scopes

### April 15
 - [ ] Re-factor to support hooks in the initial file ingest
 - [ ] Modularize

### April 15
 - [ ] Build tree per signal.
 - [ ] Each signal also comes with a value change buffer to
       avoid frequent disk readouts.

# VCD Spec Questions
- [ ] I'm pretty sure that only one statement per line is allowed.