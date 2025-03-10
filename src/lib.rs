// Copyright (C) 2022 Yehowshua Immanuel
// This program is distributed under both the GPLV3 license
// and the YEHOWSHUA license, both of which can be found at
// the root of the folder containing the sources for this program.

mod vcd;
pub use vcd::parse::parse_vcd;
pub use vcd::signal::{Signal, SignalErrors, SignalType, SignalValue};
pub use vcd::types::{Metadata, Timescale, Version};
pub use vcd::types::{ScopeIdx, SignalIdx, VCD};

pub use num::BigUint;
