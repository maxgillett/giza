use core::ops::Range;

pub use math::{fields::f128::BaseElement as Felt, FieldElement, StarkField};

pub mod word;
pub use word::{Decomposition, FieldHelpers, FlagGroupDecomposition, Word};

pub mod flags;
use flags::NUM_FLAGS;

pub mod inputs;
pub use inputs::ProgramInputs;

pub const NUM_REGISTER_COLS: usize = 3;
pub const NUM_INSTRUCTION_COLS: usize = NUM_FLAGS + 13;
pub const NUM_INSTRUCTION_AUX_COLS: usize = 2;

// TRACE LAYOUT
// -----------------------------------------------------------------------------------------
//       state            memory        range check
//    (37 columns)     (12 columns)     (4 columns)
// ├───────────────┴───────────────┴─────────────────┤

// State trace
pub const STATE_TRACE_OFFSET: usize = 0;
pub const STATE_TRACE_WIDTH: usize =
    NUM_REGISTER_COLS + NUM_INSTRUCTION_COLS + NUM_INSTRUCTION_AUX_COLS;
pub const STATE_TRACE_RANGE: Range<usize> = range(STATE_TRACE_OFFSET, STATE_TRACE_WIDTH);

// Memory trace
pub const MEM_TRACE_OFFSET: usize = STATE_TRACE_OFFSET + STATE_TRACE_WIDTH;
pub const MEM_TRACE_WIDTH: usize = 12;
pub const MEM_TRACE_RANGE: Range<usize> = range(MEM_TRACE_OFFSET, MEM_TRACE_WIDTH);

// Range check trace
pub const RC_TRACE_OFFSET: usize = MEM_TRACE_OFFSET + MEM_TRACE_WIDTH;
pub const RC_TRACE_WIDTH: usize = 3;
pub const RC_TRACE_RANGE: Range<usize> = range(RC_TRACE_OFFSET, RC_TRACE_WIDTH);

pub const TRACE_WIDTH: usize = MEM_TRACE_OFFSET + MEM_TRACE_WIDTH;

/// Returns a [Range] initialized with the specified `start` and with `end` set to `start` + `len`.
pub const fn range(start: usize, len: usize) -> Range<usize> {
    Range {
        start,
        end: start + len,
    }
}
