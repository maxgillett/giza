use crate::memory::Memory;
use crate::runner::State;
use giza_core::{
    Felt, FieldElement, StarkField, MEM_A_TRACE_RANGE, MEM_A_TRACE_WIDTH, MEM_V_TRACE_RANGE,
    OFF_X_TRACE_RANGE, TRACE_WIDTH,
};
use std::fs::File;
use std::fs;
use std::io::Read;
use hex::encode;
use std::iter;
use winterfell::{Matrix, Trace, TraceLayout};
use std::mem;

pub struct ExecutionTrace {
    layout: TraceLayout,
    meta: Vec<u8>,
    trace: Matrix<Felt>,
    public_mem: Memory,
}

fn read_binary(path: &str) -> Vec<u8> {
    let mut file = File::open(&path).expect("no file found");
    let metadata = fs::metadata(&path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    file.read(&mut buffer).expect("buffer overflow");
    buffer
}


// Memory format.
// List<MemoryItem>

// https://sourcegraph.com/github.com/starkware-libs/cairo-lang@2abd303e1808612b724bc1412b2b5babd04bb4e7/-/blob/src/starkware/cairo/lang/vm/cairo_run.py?L368:9
// 3618502788666131213697322783095070105623107215331596699973092056135872020481
// field_bytes = math.ceil(program.prime.bit_length() / 8) = 32

struct MemoryDump {
    items: Vec<MemoryItem>,
}

struct MemoryItem {
    // little endian
    address: [u8; 8],
    value: [u8; 32],
}
struct TraceItem {
    ap: [u8; 1],
    fp: [u8; 1],
    pc: [u8; 1]
}
use std::path::PathBuf;


pub fn load_trace_from_file(trace_path: PathBuf, memory_path: PathBuf) -> ExecutionTrace {
    {
        let mut f = File::open(&memory_path)
            .expect("no file found");
        let metadata = fs::metadata(&memory_path)
            .expect("unable to read metadata");
        let length = metadata.len() as usize;

        println!("Memory:");
        
        let public_mem = Memory::new(vec![]).clone();
        let mut bytes_read = 0;

        loop {
            let mut memory_item: MemoryItem = unsafe { mem::zeroed() };
            
            if bytes_read == length {
                break
            }

            bytes_read += f.read(&mut memory_item.address).unwrap();
            memory_item.address = memory_item.address
                .iter()
                .map(|x| x.to_le_bytes()[0])
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap();

            bytes_read += f.read(&mut memory_item.value).unwrap();
            
            memory_item.value = memory_item.value
                .iter()
                .map(|x| x.to_be_bytes()[0])
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap();

            // println!("{:#06x} {:#010x}", memory_item.address, memory_item.value);
            println!("{} {}", hex::encode(memory_item.address), hex::encode(memory_item.value));

            // public_mem.write(
            //     // Felt::try_from(memory_item.address).ok(),
            //     // Felt::try_from(memory_item.value).ok(),
            //     *Felt::bytes_as_elements(&memory_item.address).unwrap(),
            //     *Felt::bytes_as_elements(&memory_item.value).unwrap()
            // );
        }
    }

    {
        let mut f = File::open(&trace_path)
            .expect("no file found");
        let metadata = fs::metadata(&trace_path)
            .expect("unable to read metadata");
        let length = metadata.len() as usize;

        println!("Trace:");
        
        let mut bytes_read = 0;
        let mut i = 0;

        loop {
            let mut trace_item: TraceItem = unsafe { mem::zeroed() };
            
            if bytes_read == length {
                break
            }

            bytes_read += f.read(&mut trace_item.ap).unwrap();
            bytes_read += f.read(&mut trace_item.fp).unwrap();
            bytes_read += f.read(&mut trace_item.pc).unwrap();

            // println!("{:#06x} {:#010x}", memory_item.address, memory_item.value);
            println!(
                "{:#04x} ap={} fp={} pc={}", 
                i,
                hex::encode(trace_item.ap), 
                hex::encode(trace_item.fp),
                hex::encode(trace_item.pc),
            );

            i += 1;

            // public_mem.write(
            //     // Felt::try_from(memory_item.address).ok(),
            //     // Felt::try_from(memory_item.value).ok(),
            //     *Felt::bytes_as_elements(&memory_item.address).unwrap(),
            //     *Felt::bytes_as_elements(&memory_item.value).unwrap()
            // );
        }
    }
    

    // let trace = 

    // TODO.
    ExecutionTrace {
        layout: TraceLayout::new(
            TRACE_WIDTH,
            [12], // aux_segment widths
            [2],  // aux_segment rands
        ),
        meta: Vec::new(),
        trace: Matrix::new(vec![]),
        // public_mem: public_mem,
        public_mem: Memory::new(vec![]).clone(),
    }
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
    trace_len: usize,
}

impl<'a, E: FieldElement> Layouter<'a, E> {
    fn new(columns: &'a mut Vec<Vec<E>>, frame_len: usize, trace_len: usize) -> Self {
        Self {
            columns,
            frame_len,
            trace_len,
        }
    }

    /// Add one or more columns to the trace. The chunk size determines the number
    /// of subcolumn elements to place within each frame chunk (defaults to 1)
    /// starting from the top most row of the chunk.
    /// Panics if chunk_size is greater than frame_len
    fn add_columns(&mut self, subcols: &[Vec<E>], chunk_size: Option<usize>) {
        for subcol in subcols.iter() {
            let mut col = E::zeroed_vector(self.frame_len * self.trace_len);
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
    fn add_virtual_columns(&mut self, vcols: &[VirtualColumn<E>]) {
        for vcol in vcols.iter() {
            let subcol = vcol.to_column();
            self.add_columns(&[subcol], Some(vcol.subcols.len()));
        }
    }

    /// Resize columns to next power of two
    fn resize_all(&mut self) {
        let trace_len_pow2 = self.columns[0].len().next_power_of_two();
        for column in self.columns.iter_mut() {
            column.truncate(self.trace_len);
            let last_value = column.last().copied().unwrap();
            column.resize(trace_len_pow2, last_value);
        }
    }
}

impl ExecutionTrace {
    /// Builds an execution trace
    pub(super) fn new(num_steps: usize, state: &mut State, public_mem: &Memory) -> Self {
        // TODO: Don't hardcode index values here
        let mut t0 = vec![];
        let mut t1 = vec![];
        for step in 0..num_steps {
            t0.push(state.flags[9][step] * state.mem_v[1][step]); // f_pc_jnz * dst
            t1.push(t0[step] * state.res[0][step]); // t_0 * res
        }

        // Append dummy (0,0) public memory values to mem_a and mem_v
        let zero_column = vec![Felt::ZERO; public_mem.size() as usize - 1];
        for (n, col) in VirtualColumn::new(&[zero_column])
            .to_columns(&[MEM_A_TRACE_WIDTH])
            .iter()
            .enumerate()
        {
            state.mem_a[n].extend(col);
            state.mem_v[n].extend(col);
        }

        // Layout the trace
        let mut columns: Vec<Vec<Felt>> = Vec::with_capacity(TRACE_WIDTH);
        let mut layouter = Layouter::new(&mut columns, 1, num_steps);
        layouter.add_columns(&state.flags, None);
        layouter.add_columns(&state.res, None);
        layouter.add_columns(&state.mem_p, None);
        layouter.add_columns(&state.mem_a, None);
        layouter.add_columns(&state.mem_v, None);
        layouter.add_columns(&state.offsets, None);
        layouter.add_columns(&[t0, t1], None);

        layouter.resize_all();

        Self {
            // TODO: Enable support in Winterfell for additional aux segments
            layout: TraceLayout::new(
                TRACE_WIDTH,
                [12], // aux_segment widths
                [2],  // aux_segment rands
            ),
            meta: Vec::new(),
            trace: Matrix::new(columns),
            public_mem: public_mem.clone(),
        }
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
            //1 => build_aux_segment_rc(self, rand_elements),
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
    let mut a_replaced = a.clone();
    let mut v_replaced = v.clone();
    let len = a_replaced.len() - trace.public_mem.size() as usize - 1;
    a_replaced.truncate(len);
    v_replaced.truncate(len);
    a_replaced.extend(
        (0..trace.public_mem.size() - 1)
            .map(|x| Felt::from(x))
            .collect::<Vec<Felt>>(),
    );
    v_replaced.extend(
        trace
            .public_mem
            .data
            .iter()
            .map(|x| x.unwrap().word().into())
            .collect::<Vec<Felt>>(),
    );

    // Construct two additional virtual columns sorted by memory access
    let mut indices = (0..a_replaced.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| a_replaced[i].as_int());
    let a_prime = indices
        .iter()
        .map(|x| a_replaced[*x].into())
        .collect::<Vec<E>>();
    let v_prime = indices
        .iter()
        .map(|x| v_replaced[*x].into())
        .collect::<Vec<E>>();

    // Compute virtual column of permutation products
    let mut p = vec![E::ONE; trace.length() * MEM_A_TRACE_WIDTH];
    let p_len = p.len();
    for i in 0..p_len - 2 {
        let a_i: E = a[i].into();
        let v_i: E = v[i].into();
        p[i + 1] = (z - (a_i + alpha * v_i).into()) * p[i]
            / (z - (a_prime[i] + alpha * v_prime[i]).into());
    }
    p[p_len - 1] = E::ONE;

    // Split virtual columns into separate auxiliary columns
    let mut aux_columns = VirtualColumn::new(&[a_prime, v_prime, p]).to_columns(&[4, 4, 4]);

    // Resize auxiliary columns to next power of two
    let trace_len_pow2 = aux_columns
        .iter()
        .map(|x| x.len().next_power_of_two())
        .max()
        .unwrap();
    for column in aux_columns.iter_mut() {
        let last_value = column.last().copied().unwrap();
        column.resize(trace_len_pow2, last_value);
    }

    Some(Matrix::new(aux_columns))
}

/// Write documentation
fn build_aux_segment_rc<E>(trace: &ExecutionTrace, rand_elements: &[E]) -> Option<Matrix<E>>
where
    E: FieldElement + From<Felt>,
{
    let z = rand_elements[2];

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

    // TODO: We need to add unused/blank main and aux trace cells to fill gaps in the range check
    // This should be done when laying out the main trace

    let mut p = vec![E::ONE; trace.length()];
    for i in 0..a.len() - 2 {
        let a_i: E = a[i].into();
        p[i + 1] = (z - a_i) * p[i] / (z - a_i);
    }

    // Split virtual columns into separate auxiliary columns
    let aux_columns = VirtualColumn::new(&[a_prime, p]).to_columns(&[3, 3]);
    Some(Matrix::new(aux_columns))
}
