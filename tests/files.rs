// Copyright (C) 2022 Yehowshua Immanuel
// This program is distributed under both the GPLV3 license
// and the YEHOWSHUA license, both of which can be found at
// the root of the folder containing the sources for this program.

// TODO: we should eventually be able to only test on just
// the files const
pub const FILES : [&str; 30] = [
    "./tests/vcd-files/aldec/SPI_Write.vcd",
    "./tests/vcd-files/ghdl/alu.vcd",
    "./tests/vcd-files/ghdl/idea.vcd",
    "./tests/vcd-files/ghdl/pcpu.vcd",
    "./tests/vcd-files/gtkwave-analyzer/perm_current.vcd",
    "./tests/vcd-files/icarus/CPU.vcd",
    "./tests/vcd-files/icarus/rv32_soc_TB.vcd",
    "./tests/vcd-files/icarus/test1.vcd",
    "./tests/vcd-files/model-sim/CPU_Design.msim.vcd",
    "./tests/vcd-files/model-sim/clkdiv2n_tb.vcd",
    "./tests/vcd-files/my-hdl/Simple_Memory.vcd",
    "./tests/vcd-files/my-hdl/sigmoid_tb.vcd",
    "./tests/vcd-files/my-hdl/top.vcd",
    "./tests/vcd-files/ncsim/ffdiv_32bit_tb.vcd",
    "./tests/vcd-files/quartus/mipsHardware.vcd",
    "./tests/vcd-files/quartus/wave_registradores.vcd",
    "./tests/vcd-files/questa-sim/dump.vcd",
    "./tests/vcd-files/questa-sim/test.vcd",
    "./tests/vcd-files/riviera-pro/dump.vcd",
    "./tests/vcd-files/systemc/waveform.vcd",
    "./tests/vcd-files/treadle/GCD.vcd",
    "./tests/vcd-files/vcs/Apb_slave_uvm_new.vcd",
    "./tests/vcd-files/vcs/datapath_log.vcd",
    "./tests/vcd-files/vcs/processor.vcd",
    "./tests/vcd-files/verilator/swerv1.vcd",
    "./tests/vcd-files/verilator/vlt_dump.vcd",
    "./tests/vcd-files/vivado/iladata.vcd",
    "./tests/vcd-files/xilinx_isim/test.vcd",
    "./tests/vcd-files/xilinx_isim/test1.vcd",
    // TODO : add signal ignore list to handle bitwidth mismatches
    "./tests/vcd-files/xilinx_isim/test2x2_regex22_string1.vcd"
];

pub const GOOD_DATE_FILES : [&str; 24] = [
    "./test-vcd-files/aldec/SPI_Write.vcd",
    "./test-vcd-files/ghdl/alu.vcd",
    "./test-vcd-files/ghdl/idea.vcd",
    "./test-vcd-files/ghdl/pcpu.vcd",
    "./test-vcd-files/gtkwave-analyzer/perm_current.vcd",
    "./test-vcd-files/icarus/CPU.vcd",
    "./test-vcd-files/icarus/rv32_soc_TB.vcd",
    "./test-vcd-files/icarus/test1.vcd",
    "./test-vcd-files/model-sim/CPU_Design.msim.vcd",
    "./test-vcd-files/model-sim/clkdiv2n_tb.vcd",
    "./test-vcd-files/my-hdl/Simple_Memory.vcd",
    "./test-vcd-files/my-hdl/sigmoid_tb.vcd",
    "./test-vcd-files/my-hdl/top.vcd",
    "./test-vcd-files/questa-sim/dump.vcd",
    "./test-vcd-files/questa-sim/test.vcd",
    "./test-vcd-files/riviera-pro/dump.vcd",
    "./test-vcd-files/vcs/Apb_slave_uvm_new.vcd",
    "./test-vcd-files/vcs/datapath_log.vcd",
    "./test-vcd-files/vcs/processor.vcd",
    "./test-vcd-files/verilator/swerv1.vcd",
    "./test-vcd-files/verilator/vlt_dump.vcd",
    "./test-vcd-files/xilinx_isim/test.vcd",
    "./test-vcd-files/xilinx_isim/test1.vcd",
    "./test-vcd-files/xilinx_isim/test2x2_regex22_string1.vcd"
];

pub const BAD_DATE_FILES : [&str; 6] = [
    "./test-vcd-files/ncsim/ffdiv_32bit_tb.vcd",
    "./test-vcd-files/quartus/mipsHardware.vcd",
    "./test-vcd-files/quartus/wave_registradores.vcd",
    "./test-vcd-files/systemc/waveform.vcd",
    "./test-vcd-files/treadle/GCD.vcd",
    "./test-vcd-files/vivado/iladata.vcd",
];