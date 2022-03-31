use super::{MemoryTrace, StateTrace};
use crate::memory::Memory;
use crate::runner::State;
use giza_core::{
    Felt, MEM_TRACE_OFFSET, MEM_TRACE_RANGE, STATE_TRACE_OFFSET, STATE_TRACE_RANGE, TRACE_WIDTH,
};

use winterfell::Trace;

pub struct ExecutionTrace {
    meta: Vec<u8>,
    memory: MemoryTrace,
    state: StateTrace,
}

impl ExecutionTrace {
    /// Builds an execution trace from the memory and state
    pub(super) fn new(num_steps: usize, memory: &Memory, state: &State) -> Self {
        let mut memory_trace = memory.into_trace();
        let mut state_trace = state.into_trace();
        let trace_len = vec![memory_trace.len(), state_trace.len()]
            .iter()
            .max()
            .unwrap()
            .next_power_of_two();
        for column in memory_trace.iter_mut() {
            column.truncate(num_steps);
            let last_value = column.last().copied().unwrap();
            column.resize(trace_len, last_value);
        }
        for column in state_trace.iter_mut() {
            column.truncate(num_steps);
            let last_value = column.last().copied().unwrap();
            column.resize(trace_len, last_value);
        }
        //    for j in 0..trace_len {
        //        print!("{:?} ", state_trace[i][j].as_int());
        //    }
        //    println!("");
        //}
        Self {
            meta: Vec::new(),
            memory: memory_trace,
            state: state_trace,
        }
    }
}

impl Trace for ExecutionTrace {
    type BaseField = Felt;

    fn width(&self) -> usize {
        TRACE_WIDTH
    }

    fn length(&self) -> usize {
        self.state[0].len()
    }

    fn get(&self, col_idx: usize, row_idx: usize) -> Felt {
        match col_idx {
            i if STATE_TRACE_RANGE.contains(&i) => self.state[i - STATE_TRACE_OFFSET][row_idx],
            i if MEM_TRACE_RANGE.contains(&i) => self.memory[i - MEM_TRACE_OFFSET][row_idx],
            _ => panic!("invalid column index"),
        }
    }

    fn meta(&self) -> &[u8] {
        &self.meta
    }

    fn read_row_into(&self, step: usize, target: &mut [Felt]) {
        for (i, column) in self.state.iter().enumerate() {
            target[i + STATE_TRACE_OFFSET] = column[step];
        }
        for (i, column) in self.memory.iter().enumerate() {
            target[i + MEM_TRACE_OFFSET] = column[step];
        }
    }

    fn into_columns(self) -> Vec<Vec<Felt>> {
        let mut result: Vec<Vec<Felt>> = self.state.into();
        self.memory.into_iter().for_each(|v| result.push(v));
        result
    }
}
