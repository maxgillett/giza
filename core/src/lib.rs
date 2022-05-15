use core::ops::Range;

pub use math::{fields::f128::BaseElement as Felt, ExtensionOf, FieldElement, StarkField};

pub mod word;
pub use word::{
    bias, FieldHelpers, FlagDecomposition, FlagGroupDecomposition, OffsetDecomposition, Word,
};

pub mod inputs;
pub use inputs::ProgramInputs;

pub mod flags;

// MAIN TRACE LAYOUT
// -----------------------------------------------------------------------------------------
//  A.  flags   (16) : Decoded instruction flags
//  B.  res     (1)  : Res value
//  C.  mem_p   (2)  : Temporary memory pointers (ap and fp)
//  D.  mem_a   (4)  : Memory addresses (pc, dst_addr, op0_addr, op1_addr)
//  E.  mem_v   (4)  : Memory values (inst, dst, op0, op1)
//  F.  offsets (3)  : (off_dst, off_op0, off_op1)
//  G.  aux     (2)  : (t0, t1)
//
//  A                B C  D    E    F
// ├xxxxxxxxxxxxxxxx|x|xx|xxxx|xxxx|xxx┤
//

pub const FLAG_TRACE_OFFSET: usize = 0;
pub const FLAG_TRACE_WIDTH: usize = 16;
pub const FLAG_TRACE_RANGE: Range<usize> = range(FLAG_TRACE_OFFSET, FLAG_TRACE_WIDTH);

pub const RES_TRACE_OFFSET: usize = 16;
pub const RES_TRACE_WIDTH: usize = 1;
pub const RES_TRACE_RANGE: Range<usize> = range(RES_TRACE_OFFSET, RES_TRACE_WIDTH);

pub const MEM_P_TRACE_OFFSET: usize = 17;
pub const MEM_P_TRACE_WIDTH: usize = 2;
pub const MEM_P_TRACE_RANGE: Range<usize> = range(MEM_P_TRACE_OFFSET, MEM_P_TRACE_WIDTH);

pub const MEM_A_TRACE_OFFSET: usize = 19;
pub const MEM_A_TRACE_WIDTH: usize = 4;
pub const MEM_A_TRACE_RANGE: Range<usize> = range(MEM_A_TRACE_OFFSET, MEM_A_TRACE_WIDTH);

pub const MEM_V_TRACE_OFFSET: usize = 23;
pub const MEM_V_TRACE_WIDTH: usize = 4;
pub const MEM_V_TRACE_RANGE: Range<usize> = range(MEM_V_TRACE_OFFSET, MEM_V_TRACE_WIDTH);

pub const OFF_X_TRACE_OFFSET: usize = 27;
pub const OFF_X_TRACE_WIDTH: usize = 3;
pub const OFF_X_TRACE_RANGE: Range<usize> = range(OFF_X_TRACE_OFFSET, OFF_X_TRACE_WIDTH);

pub const TX_TRACE_OFFSET: usize = 30;
pub const TX_TRACE_WIDTH: usize = 2;
pub const TX_TRACE_RANGE: Range<usize> = range(TX_TRACE_OFFSET, TX_TRACE_WIDTH);

pub const TRACE_WIDTH: usize = 32;

// AUX TRACE LAYOUT (Memory)
// -----------------------------------------------------------------------------------------
//  A.  a_m_prime  (4) : Sorted memory address
//  B.  v_m_prime  (4) : Sorted memory values
//  C.  p_m        (4) : Permutation product (memory)
//
//  A    B    C
// ├xxxx|xxxx|xxxx┤

pub const A_M_PRIME_OFFSET: usize = 0;
pub const A_M_PRIME_WIDTH: usize = 4;

pub const V_M_PRIME_OFFSET: usize = 4;
pub const V_M_PRIME_WIDTH: usize = 4;

pub const P_M_OFFSET: usize = 8;
pub const P_M_WIDTH: usize = 4;

// AUX TRACE LAYOUT (Range check)
// -----------------------------------------------------------------------------------------
//  D.  a_rc_prime (3) : Sorted offset values
//  E.  p_rc       (3) : Permutation product (range check)
//
//  D   E
// ├xxx|xxx┤
//

pub const A_RC_PRIME_OFFSET: usize = 12;
pub const A_RC_PRIME_WIDTH: usize = 3;

pub const P_RC_OFFSET: usize = 14;
pub const P_RC_WIDTH: usize = 3;

/// Returns a [Range] initialized with the specified `start` and with `end` set to `start` + `len`.
pub const fn range(start: usize, len: usize) -> Range<usize> {
    Range {
        start,
        end: start + len,
    }
}

/// A structure to store program counter, allocation pointer and frame pointer
#[derive(Clone, Copy, Debug)]
pub struct RegisterState {
    /// Program counter: points to address in memory
    pub pc: Felt,
    /// Allocation pointer: points to first free space in memory
    pub ap: Felt,
    /// Frame pointer: points to the beginning of the stack in memory (for arguments)
    pub fp: Felt,
}

pub struct InstructionState {
    /// Instruction
    pub inst: Word,
    pub inst_size: Felt,
    /// Addresses
    pub dst_addr: Felt,
    pub op0_addr: Felt,
    pub op1_addr: Felt,
    /// Values
    pub dst: Option<Felt>,
    pub op0: Option<Felt>,
    pub op1: Option<Felt>,
    /// Result
    pub res: Option<Felt>,
}

impl RegisterState {
    /// Creates a new triple of pointers
    pub fn new<T: Into<Felt>>(pc: T, ap: T, fp: T) -> Self {
        RegisterState {
            pc: pc.into(),
            ap: ap.into(),
            fp: fp.into(),
        }
    }
}

impl InstructionState {
    /// Creates a new set instruction word and operand state
    pub fn new(
        inst: Word,
        inst_size: Felt,
        dst: Option<Felt>,
        op0: Option<Felt>,
        op1: Option<Felt>,
        res: Option<Felt>,
        dst_addr: Felt,
        op0_addr: Felt,
        op1_addr: Felt,
    ) -> Self {
        InstructionState {
            inst,
            inst_size,
            dst,
            op0,
            op1,
            res,
            dst_addr,
            op0_addr,
            op1_addr,
        }
    }
}
