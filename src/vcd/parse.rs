use chrono::prelude::*;
use itertools::Itertools;
use std::fs::File;
use ::function_name::named;

use super::*;

mod combinator_atoms;
use combinator_atoms::*;

mod types;
use types::*;

mod metadata;
use metadata::*;


pub fn parse_vcd(file : File) {
    let mut word_gen = WordReader::new(file);

    let header = parse_metadata(&mut word_gen).unwrap();
    dbg!(header);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    #[test]
    fn headers() {
        let files = vec![
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
            // "./test-vcd-files/ncsim/ffdiv_32bit_tb.vcd",
            // "./test-vcd-files/quartus/mipsHardware.vcd",
            // "./test-vcd-files/quartus/wave_registradores.vcd",
            "./test-vcd-files/questa-sim/dump.vcd",
            "./test-vcd-files/questa-sim/test.vcd",
            "./test-vcd-files/riviera-pro/dump.vcd",
            // "./test-vcd-files/systemc/waveform.vcd",
            // "./test-vcd-files/treadle/GCD.vcd",
            "./test-vcd-files/vcs/Apb_slave_uvm_new.vcd",
            "./test-vcd-files/vcs/datapath_log.vcd",
            "./test-vcd-files/vcs/processor.vcd",
            "./test-vcd-files/verilator/swerv1.vcd",
            "./test-vcd-files/verilator/vlt_dump.vcd",
            // "./test-vcd-files/vivado/iladata.vcd",
            "./test-vcd-files/xilinx_isim/test.vcd",
            "./test-vcd-files/xilinx_isim/test1.vcd",
            "./test-vcd-files/xilinx_isim/test2x2_regex22_string1.vcd"
        ];

        for file in files {
            let metadata = parse_metadata(
                &mut WordReader::new(
                    File::open(file)
                    .unwrap()
                )
            );
            assert!(metadata.is_ok());
            assert!(metadata.unwrap().date.is_some());
        }

    }
}