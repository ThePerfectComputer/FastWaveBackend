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

# TODO
 - [x] We need a way to merge lines.
 - [ ] Need to perform signal aliasing
     - use vec of enum {Sig, Alias}
 - [ ] Should insert nodes in BFS order
 - [ ] Change states to lowercase
 - [ ] We need to start regression testing the parser over all files
 - [ ] Take a look at GTKWave parser to compare effificiency.
 - [ ] Send survey to community channel.

### May 18
 - [ ] move while loop into word yielding iterator

# Files
 - ./test-vcd-files/aldec/SPI_Write.vcd
 - ./test-vcd-files/ghdl/alu.vcd
 - ./test-vcd-files/ghdl/idea.vcd
 - ./test-vcd-files/ghdl/pcpu.vcd
 - ./test-vcd-files/gtkwave-analyzer/perm_current.vcd
 - ./test-vcd-files/icarus/CPU.vcd
 - ./test-vcd-files/icarus/rv32_soc_TB.vcd
 - ./test-vcd-files/icarus/test1.vcd
 - ./test-vcd-files/model-sim/CPU_Design.msim.vcd
 - ./test-vcd-files/model-sim/clkdiv2n_tb.vcd
 - ./test-vcd-files/my-hdl/Simple_Memory.vcd
 - ./test-vcd-files/my-hdl/sigmoid_tb.vcd
 - ./test-vcd-files/my-hdl/top.vcd
 - ./test-vcd-files/ncsim/ffdiv_32bit_tb.vcd
 - ./test-vcd-files/quartus/mipsHardware.vcd
 - ./test-vcd-files/quartus/wave_registradores.vcd
 - ./test-vcd-files/questa-sim/dump.vcd
 - ./test-vcd-files/questa-sim/test.vcd
 - ./test-vcd-files/riviera-pro/dump.vcd
 - ./test-vcd-files/systemc/waveform.vcd
 - ./test-vcd-files/treadle/GCD.vcd
 - ./test-vcd-files/vcs/Apb_slave_uvm_new.vcd
 - ./test-vcd-files/vcs/datapath_log.vcd
 - ./test-vcd-files/vcs/processor.vcd
 - ./test-vcd-files/verilator/swerv1.vcd
 - ./test-vcd-files/verilator/vlt_dump.vcd
 - ./test-vcd-files/vivado/iladata.vcd
 - ./test-vcd-files/xilinx_isim/test.vcd
 - ./test-vcd-files/xilinx_isim/test1.vcd
 - ./test-vcd-files/xilinx_isim/test2x2_regex22_string1.vcd