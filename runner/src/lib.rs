use giza_core::{Felt, MEM_TRACE_WIDTH, STATE_TRACE_WIDTH};

pub mod memory;
pub use memory::Memory;

pub mod runner;
pub use runner::Program;

pub mod hints;

mod trace;
pub use trace::ExecutionTrace;

mod errors;
pub use errors::ExecutionError;

type StateTrace = [Vec<Felt>; STATE_TRACE_WIDTH];
type MemoryTrace = [Vec<Felt>; MEM_TRACE_WIDTH];
