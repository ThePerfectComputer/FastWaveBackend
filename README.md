Copyright - Yehowshua Immanuel

# A High performance, VCD Parser written in Rust

## Current Features

FastWave currently offers highly robust error(at least on the sample 
VCD files in this repository) handling which GTKWave doesn't have. For
eample, selecting the ``UTILIZATON_ENABLED`` after opening 
[test2x2_regex22_string1.vcd](./test-vcd-files/xilinx_isim/test2x2_regex22_string1.vcd),
(one of the sample xilinx vcd test files) in GtkWave, will crash GtkWave since
this signal is malformed. FastWave on the otherhand simply informs you the
signal is malformed.

## Performance

### Methods
Below is a table of performance comparisons on a large 3.04GB VCD file
that can be found 
[here](https://drive.google.com/file/d/1pfm2qo2l8fGTHHJ8TLrg1vSGaV_TUbp2/view?usp=sharing).

For getting the GtkWave results, I fired up GtkWave, clicke on 
``File``->``Open New Window``, and selected the large VCD file.
I started my stopwatch as soon as I pressed enter to beging loading the VCD
file, and stopped my stopwatch once the GtkWave titlebar reached 100%.
   
To get the memory consumption, I opened Activity Monitor on Mac, and recorded
the GtkWave memory usage before and after loading the large VCD file, and 
took the difference.

I noticed that when running FastWave in the VsCode terminal as opposed
to the MacOS system terminal or the Lapce terminal, FastWave is notably
slower.

### Results

| Software | Time(s) | Memory(MB) |
|----------|---------|------------|
| GtkWave  | ~30     | 89.8       |
| FastWave | 15.09   | 267.3      |


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

# Testing on Bad Files
You may wish to test the parser on a malformed VCD just to make
sure that the parser gives useful/sane errors.

Here's a command to test on a malformed VCD:
`cargo run --release test-vcd-files/VCD_file_with_errors.vcd`


# TODO

## Features
 - [ ] be explicit with imports, remove exports as possible
       once FastWave is known to be fairly stable.
 - [ ] do a read through all the code
    - make contents of src/types.rs public as necessary.
 - [ ] Print out git commit or release number.
 - [ ] Take a look at GTKWave parser to compare efficiency.
 - [ ] Move part of the performance section to another markdown file.

## Repairs
 - [ ] make a custom date parser for possibly up to 18 different versions(that is, for each possible tool).
 - [ ] Consolidate error messages and add cursors throughout.
 - [ ] Fix warnings especially usage and restriction warnings once I'm
       able to successfully parse all sample VCDs.

## Code Consistency
 - [ ] Change error messages to line and filenames. Go through all calls to unwrap.
   - [ ] search for any unwraps or any direct vectors indexing
 - [ ] Handle TODOs
 - [ ] Remove debug code/comments.

## Marketing
 - [ ] Send survey to community 