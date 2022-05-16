use crate::memory::Memory;
use crate::runner::State;
use giza_core::{
    Felt, FieldElement, StarkField, Word, MEM_A_TRACE_RANGE, MEM_A_TRACE_WIDTH, MEM_V_TRACE_RANGE,
    OFF_X_TRACE_RANGE, OFF_X_TRACE_WIDTH, TRACE_WIDTH,
};
use std::collections::HashSet;

use winterfell::{Matrix, Trace, TraceLayout};

pub struct ExecutionTrace {
    layout: TraceLayout,
    meta: Vec<u8>,
    trace: Matrix<Felt>,
    pub memory: Memory,
    pub rc_min: u16,
    pub rc_max: u16,
}

/// A virtual column is composed of one or more subcolumns.
struct VirtualColumn<'a, E: FieldElement> {
    subcols: &'a [Vec<E>],
}

impl<'a, E: FieldElement> VirtualColumn<'a, E> {
    fn new(subcols: &'a [Vec<E>]) -> Self {
        Self { subcols }
    }

    /// Pack subcolumns into a single column: cycle through each subcolumn, appending
    /// a single value to the column for each iteration step until exhausted
    fn to_column(&self) -> Vec<E> {
        let mut col: Vec<E> = vec![];
        for n in 0..self.subcols[0].len() {
            for subcol in self.subcols {
                col.push(subcol[n]);
            }
        }
        col
    }

    /// Split subcolumns into multiple columns: cycle through each subcolumn, appending...
    fn to_columns(&self, num_rows: &[usize]) -> Vec<Vec<E>> {
        let mut n = 0;
        let mut cols: Vec<Vec<E>> = vec![vec![]; num_rows.iter().sum()];
        for (subcol, width) in self.subcols.iter().zip(num_rows) {
            for (elem, idx) in subcol.iter().zip((0..*width).cycle()) {
                cols[idx + n].push(*elem);
            }
            n += width;
        }
        cols
    }
}

struct Layouter<'a, E: FieldElement> {
    columns: &'a mut Vec<Vec<E>>,
    frame_len: usize,
}

impl<'a, E: FieldElement> Layouter<'a, E> {
    fn new(columns: &'a mut Vec<Vec<E>>, frame_len: usize) -> Self {
        Self { columns, frame_len }
    }

    /// Add one or more columns to the trace. The chunk size determines the number
    /// of subcolumn elements to place within each frame chunk (defaults to 1)
    /// starting from the top most row of the chunk.
    /// Panics if chunk_size is greater than frame_len
    fn add_columns(&mut self, subcols: &[Vec<E>], chunk_size: Option<usize>) {
        for subcol in subcols.iter() {
            let mut col = E::zeroed_vector(subcol.len());
            for (col_chunk, subcol_chunk) in col
                .chunks_mut(self.frame_len)
                .zip(subcol.chunks(chunk_size.unwrap_or(1)))
            {
                for (n, elem) in subcol_chunk.iter().enumerate() {
                    col_chunk[n] = *elem
                }
            }
            self.columns.push(col);
        }
    }

    /// Add one or more virtual columns to the trace
    #[allow(dead_code)]
    fn add_virtual_columns(&mut self, vcols: &[VirtualColumn<E>]) {
        for vcol in vcols.iter() {
            let subcol = vcol.to_column();
            self.add_columns(&[subcol], Some(vcol.subcols.len()));
        }
    }

    /// Resize columns to next power of two
    fn resize_all(&mut self) {
        let trace_len_pow2 = self
            .columns
            .iter()
            .map(|x| x.len().next_power_of_two())
            .max()
            .unwrap();
        for column in self.columns.iter_mut() {
            let last_value = column.last().copied().unwrap();
            column.resize(trace_len_pow2, last_value);
        }
    }
}

impl ExecutionTrace {
    /// Builds an execution trace
    pub(super) fn new(num_steps: usize, state: &mut State, memory: &Memory) -> Self {
        // TODO: Don't hardcode index values here
        let mut t0 = vec![];
        let mut t1 = vec![];
        for step in 0..num_steps {
            t0.push(state.flags[9][step] * state.mem_v[1][step]); // f_pc_jnz * dst
            t1.push(t0[step] * state.res[0][step]); // t_0 * res
        }

        // Append dummy (0,0) public memory values to mem_a and mem_v
        let zero_column = vec![Felt::ZERO; memory.get_codelen()];
        for (n, col) in VirtualColumn::new(&[zero_column])
            .to_columns(&[MEM_A_TRACE_WIDTH])
            .iter()
            .enumerate()
        {
            state.mem_a[n].extend(col);
            state.mem_v[n].extend(col);
        }

        // Append trace cells to offsets to fill in gaps between rc_min and rc_max
        // after biasing the offset values
        let b15 = Felt::new(2).exp(15);
        let mut rc_column = VirtualColumn::new(&state.offsets)
            .to_column()
            .into_iter()
            .map(|x| x + b15)
            .collect::<Vec<_>>();
        let rc_min = rc_column.iter().map(|x| x.as_int() as u16).min().unwrap();
        let rc_max = rc_column.iter().map(|x| x.as_int() as u16).max().unwrap();
        for x in rc_min..rc_max {
            if !rc_column.contains(&x.into()) {
                rc_column.push(x.into());
            }
        }
        let offsets = VirtualColumn::new(&[rc_column]).to_columns(&[3]);

        // Layout the trace
        let mut columns: Vec<Vec<Felt>> = Vec::with_capacity(TRACE_WIDTH);
        let mut layouter = Layouter::new(&mut columns, 1);
        layouter.add_columns(&state.flags, None);
        layouter.add_columns(&state.res, None);
        layouter.add_columns(&state.mem_p, None);
        layouter.add_columns(&state.mem_a, None);
        layouter.add_columns(&state.mem_v, None);
        layouter.add_columns(&offsets, None);
        layouter.add_columns(&[t0, t1], None);

        // TODO: When resizing mem_a and mem_v, extend execution accesses to the required
        // length (and not the appended public memory accesses). Otherwise the final value
        // cumulative product constraint may not match

        layouter.resize_all();

        Self {
            // TODO: Enable support in Winterfell for additional aux segments
            layout: TraceLayout::new(
                TRACE_WIDTH,
                &[12, 6], // aux_segment widths
                &[2, 1],  // aux_segment rands
            ),
            meta: Vec::new(),
            trace: Matrix::new(columns),
            memory: memory.clone(),
            rc_min,
            rc_max,
        }
    }

    /// Return the public memory
    pub fn public_mem(&self) -> Vec<Option<Word>> {
        self.memory.data[..self.memory.get_codelen()].to_vec()
    }
}

impl Trace for ExecutionTrace {
    type BaseField = Felt;

    fn layout(&self) -> &TraceLayout {
        &self.layout
    }

    fn length(&self) -> usize {
        self.trace.num_rows()
    }

    fn meta(&self) -> &[u8] {
        &self.meta
    }

    fn main_segment(&self) -> &Matrix<Felt> {
        &self.trace
    }

    fn build_aux_segment<E>(
        &mut self,
        aux_segments: &[Matrix<E>],
        rand_elements: &[E],
    ) -> Option<Matrix<E>>
    where
        E: FieldElement<BaseField = Self::BaseField>,
    {
        match aux_segments.len() {
            0 => build_aux_segment_mem(self, rand_elements),
            1 => build_aux_segment_rc(self, rand_elements),
            _ => None,
        }
    }
}

/// Write documentation
fn build_aux_segment_mem<E>(trace: &ExecutionTrace, rand_elements: &[E]) -> Option<Matrix<E>>
where
    E: FieldElement + From<Felt>,
{
    let z = rand_elements[0];
    let alpha = rand_elements[1];

    // Pack main trace columns into virtual columns
    let main = trace.main_segment();
    let cols_a = MEM_A_TRACE_RANGE
        .map(|i| main.get_column(i).to_vec())
        .collect::<Vec<_>>();
    let cols_v = MEM_V_TRACE_RANGE
        .map(|i| main.get_column(i).to_vec())
        .collect::<Vec<_>>();
    let a = VirtualColumn::new(&cols_a[..]).to_column();
    let v = VirtualColumn::new(&cols_v[..]).to_column();

    // Replace dummy public memory accesses
    let trace_len = a.len() - trace.memory.get_codelen();
    let mut a_replaced = a.clone()[0..trace_len].to_vec();
    let mut v_replaced = v.clone()[0..trace_len].to_vec();
    for (i, x) in trace.public_mem().iter().enumerate() {
        a_replaced.push(Felt::from(i as u64));
        v_replaced.push(x.unwrap().word().into());
    }

    // Construct two additional virtual columns sorted by memory access
    let mut indices = (0..a_replaced.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| a_replaced[i].as_int());
    let mut a_prime = vec![E::ZERO; indices.len()];
    let mut v_prime = vec![E::ZERO; indices.len()];
    for (i, j) in indices.iter().copied().enumerate() {
        a_prime[i] = a_replaced[j].into();
        v_prime[i] = v_replaced[j].into();
    }

    // Compute virtual column of permutation products
    let mut p = vec![E::ONE; trace.length() * MEM_A_TRACE_WIDTH];
    let a_0: E = a[0].into();
    let v_0: E = v[0].into();
    p[0] = (z - (a_0 + alpha * v_0).into()) / (z - (a_prime[0] + alpha * v_prime[0]).into());
    for i in 1..p.len() {
        let a_i: E = a[i].into();
        let v_i: E = v[i].into();
        p[i] = (z - (a_i + alpha * v_i).into()) * p[i - 1]
            / (z - (a_prime[i] + alpha * v_prime[i]).into());
    }

    // Split virtual columns into separate auxiliary columns
    let mut aux_columns = VirtualColumn::new(&[a_prime, v_prime, p]).to_columns(&[4, 4, 4]);
    resize_to_pow2(&mut aux_columns);

    Some(Matrix::new(aux_columns))
}

/// Write documentation
fn build_aux_segment_rc<E>(trace: &ExecutionTrace, rand_elements: &[E]) -> Option<Matrix<E>>
where
    E: FieldElement + From<Felt>,
{
    let z = rand_elements[0];

    let main = trace.main_segment();
    let cols_a = OFF_X_TRACE_RANGE
        .map(|i| main.get_column(i).to_vec())
        .collect::<Vec<_>>();

    // Pack main trace columns into virtual columns
    let a = VirtualColumn::new(&cols_a[..]).to_column();

    // Construct two additional virtual columns sorted by offset values
    let mut indices = (0..a.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| a[i].as_int());
    let a_prime = indices.iter().map(|x| a[*x].into()).collect::<Vec<E>>();

    // Compute virtual column of permutation products
    let mut p = vec![E::ONE; trace.length() * OFF_X_TRACE_WIDTH];
    let a_0: E = a[0].into();
    p[0] = (z - a_0) / (z - a_prime[0]);
    for i in 1..p.len() {
        let a_i: E = a[i].into();
        p[i] = (z - a_i) * p[i - 1] / (z - a_prime[i]);
    }

    // Split virtual columns into separate auxiliary columns
    let mut aux_columns = VirtualColumn::new(&[a_prime, p]).to_columns(&[3, 3]);
    resize_to_pow2(&mut aux_columns);

    Some(Matrix::new(aux_columns))
}

/// Resize columns to next power of two
fn resize_to_pow2<E: FieldElement>(columns: &mut [Vec<E>]) {
    let trace_len_pow2 = columns
        .iter()
        .map(|x| x.len().next_power_of_two())
        .max()
        .unwrap();
    for column in columns.iter_mut() {
        let last_value = column.last().copied().unwrap();
        column.resize(trace_len_pow2, last_value);
    }
}
