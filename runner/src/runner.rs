// Modified from https://github.com/o1-labs/proof-systems

use crate::errors::ExecutionError;
use crate::hints::{ExecutionEffect as HintExecutionEffect, HintManager};
use crate::memory::Memory;
use crate::trace::ExecutionTrace;
use giza_core::{
    flags::*, Decomposition, Felt, FieldElement, FlagGroupDecomposition, StarkField, Word,
    NUM_INSTRUCTION_COLS, NUM_REGISTER_COLS, STATE_TRACE_WIDTH,
};

use std::convert::TryInto;

/// A structure to store program counter, allocation pointer and frame pointer
#[derive(Clone, Copy)]
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
    inst: Word,
    inst_size: Felt,
    /// Operand offsets
    dst_addr: Felt,
    op0_addr: Felt,
    op1_addr: Felt,
    /// Offset values
    dst: Option<Felt>,
    op0: Option<Felt>,
    op1: Option<Felt>,
    res: Option<Felt>,
}

impl RegisterState {
    /// Creates a new triple of pointers
    pub fn new(pc: Felt, ap: Felt, fp: Felt) -> Self {
        RegisterState { pc, ap, fp }
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

/// A data structure to store a current step of computation
pub struct Step<'a> {
    pub mem: &'a mut Memory,
    hints: Option<&'a HintManager>,
    pub curr: RegisterState,
    next: Option<RegisterState>,
}

impl<'a> Step<'a> {
    /// Creates a new execution step from a step index, a word, and current pointers
    pub fn new(
        mem: &'a mut Memory,
        hints: Option<&'a HintManager>,
        ptrs: RegisterState,
    ) -> Step<'a> {
        Step {
            mem,
            hints,
            curr: ptrs,
            next: None,
        }
    }

    /// This function returns the current word instruction being executed
    fn inst(&mut self) -> Word {
        Word::new(self.mem.read(self.curr.pc).expect("pc points to None cell"))
    }

    fn apply_hint_effects(&mut self, res: HintExecutionEffect) {
        self.curr.pc = res.pc;
        self.curr.ap = res.ap;
        self.curr.fp = res.fp;
        if let Some(updates) = res.mem_updates {
            for (addr, elem) in updates.0.iter() {
                self.mem.write(Felt::from(*addr), elem.word());
            }
        }
    }

    /// Executes a step from the current registers and returns the instruction state
    pub fn execute(&mut self) -> InstructionState {
        // Execute hints and apply changes
        if let Some(manager) = self.hints {
            for hint in manager.get_hints(self.curr.pc).into_iter().flatten() {
                let changes = hint.exec(&self).unwrap();
                self.apply_hint_effects(changes);
            }
        }

        // Execute instruction
        let (op0_addr, mut op0) = self.set_op0();
        let (op1_addr, mut op1, size) = self.set_op1(op0);
        let mut res = self.set_res(op0, op1);
        let (dst_addr, mut dst) = self.set_dst();
        let next_pc = self.next_pc(size, res, dst, op1);
        let (next_ap, next_fp, op0_update, op1_update, res_update, dst_update) =
            self.next_apfp(size, res, dst, dst_addr, op1_addr);
        if op0_update.is_some() {
            op0 = op0_update;
        }
        if op1_update.is_some() {
            op1 = op1_update;
        }
        if res_update.is_some() {
            res = res_update;
        }
        if dst_update.is_some() {
            dst = dst_update;
        }
        self.next = Some(RegisterState::new(
            next_pc.expect("Empty next program counter"),
            next_ap.expect("Empty next allocation pointer"),
            next_fp.expect("Empty next frame pointer"),
        ));
        InstructionState::new(
            self.inst(),
            size,
            dst,
            op0,
            op1,
            res,
            dst_addr,
            op0_addr,
            op1_addr,
        )
    }

    /// This function computes the first operand address.
    /// Outputs: `(op0_addr, op0)`
    fn set_op0(&mut self) -> (Felt, Option<Felt>) {
        let reg = match self.inst().op0_reg() {
            /*0*/ OP0_AP => self.curr.ap, // reads first word from allocated memory
            /*1*/ _ => self.curr.fp, // reads first word from input stack
        };
        let op0_addr = reg + self.inst().off_op0();
        let op0 = self.mem.read(op0_addr);
        (op0_addr, op0)
    }

    /// This function computes the second operand address and content and the instruction size
    /// Panics if the flagset `OP1_SRC` has more than 1 nonzero bit
    /// Inputs: `op0`
    /// Outputs: `(op1_addr, op1, size)`
    fn set_op1(&mut self, op0: Option<Felt>) -> (Felt, Option<Felt>, Felt) {
        let (reg, size) = match self.inst().op1_src() {
            /*0*/
            OP1_DBL => (op0.expect("None op0 for OP1_DBL"), Felt::new(1)), // double indexing, op0 should be positive for address
            /*1*/
            OP1_VAL => (self.curr.pc, Felt::from(2u32)), // off_op1 will be 1 and then op1 contains an immediate value
            /*2*/ OP1_FP => (self.curr.fp, Felt::new(1)),
            /*4*/ OP1_AP => (self.curr.ap, Felt::new(1)),
            _ => panic!("Invalid op1_src flagset"),
        };
        let op1_addr = reg + self.inst().off_op1(); // apply second offset to corresponding register
        let op1 = self.mem.read(op1_addr);
        (op1_addr, op1, size)
    }

    /// This function computes the value of the result of the arithmetic operation
    /// Panics if a `jnz` instruction is used with an invalid format
    ///     or if the flagset `RES_LOG` has more than 1 nonzero bit
    /// Inputs: `op0`, `op1`
    /// Outputs: `res`
    fn set_res(&mut self, op0: Option<Felt>, op1: Option<Felt>) -> Option<Felt> {
        let res;
        if self.inst().pc_up() == PC_JNZ {
            /*4*/
            // jnz instruction
            if self.inst().res_log() == RES_ONE /*0*/
                && self.inst().opcode() == OPC_JMP_INC /*0*/
                && self.inst().ap_up() != AP_ADD
            /* not 1*/
            {
                res = Some(Felt::new(0)); // "unused"
            } else {
                panic!("Invalid JNZ instruction");
            }
        } else if self.inst().pc_up() == PC_SIZ /*0*/
            || self.inst().pc_up() == PC_ABS /*1*/
            || self.inst().pc_up() == PC_REL
        /*2*/
        {
            // rest of types of updates
            // common increase || absolute jump || relative jump
            res = {
                match self.inst().res_log() {
                    /*0*/
                    RES_ONE => op1, // right part is single operand
                    /*1*/
                    RES_ADD => Some(
                        op0.expect("None op0 after RES_ADD") + op1.expect("None op1 after RES_ADD"),
                    ), // right part is addition
                    /*2*/
                    RES_MUL => Some(
                        op0.expect("None op0 after RES_MUL") * op1.expect("None op1 after RES_MUL"),
                    ), // right part is multiplication
                    _ => panic!("Invalid res_log flagset"),
                }
            };
        } else {
            // multiple bits take value 1
            panic!("Invalid pc_up flagset");
        }
        res
    }

    /// This function computes the destination address
    /// Outputs: `(dst_addr, dst)`
    fn set_dst(&mut self) -> (Felt, Option<Felt>) {
        let reg = match self.inst().dst_reg() {
            /*0*/ DST_AP => self.curr.ap, // read from stack
            /*1*/ _ => self.curr.fp, // read from parameters
        };
        let dst_addr = reg + self.inst().off_dst();
        let dst = self.mem.read(dst_addr);
        (dst_addr, dst)
    }

    /// This function computes the next program counter
    /// Panics if the flagset `PC_UP` has more than 1 nonzero bit
    /// Inputs: `size`, `res`, `dst`, `op1`,
    /// Outputs: `next_pc`
    fn next_pc(
        &mut self,
        size: Felt,
        res: Option<Felt>,
        dst: Option<Felt>,
        op1: Option<Felt>,
    ) -> Option<Felt> {
        match self.inst().pc_up() {
            /*0*/
            PC_SIZ => Some(self.curr.pc + size), // common case, next instruction is right after the current one
            /*1*/
            PC_ABS => Some(res.expect("None res after PC_ABS")), // absolute jump, next instruction is in res,
            /*2*/
            PC_REL => Some(self.curr.pc + res.expect("None res after PC_REL")), // relative jump, go to some address relative to pc
            /*4*/
            PC_JNZ => {
                // conditional relative jump (jnz)
                if dst == Some(Felt::new(0)) {
                    // if condition false, common case
                    Some(self.curr.pc + size)
                } else {
                    // if condition true, relative jump with second operand
                    Some(self.curr.pc + op1.expect("None op1 after PC_JNZ"))
                }
            }
            _ => panic!("Invalid pc_up flagset"),
        }
    }

    /// This function computes the next values of the allocation and frame pointers
    /// Panics if in a `call` instruction the flagset [AP_UP] is incorrect
    ///     or if in any other instruction the flagset AP_UP has more than 1 nonzero bit
    ///     or if the flagset `OPCODE` has more than 1 nonzero bit
    /// Inputs: `size`, `res`, `dst`, `dst_addr`, `op1_addr`
    /// Outputs: `(next_ap, next_fp, op0_update, op1_update, res_update, dst_update)`
    fn next_apfp(
        &mut self,
        size: Felt,
        res: Option<Felt>,
        dst: Option<Felt>,
        dst_addr: Felt,
        op1_addr: Felt,
    ) -> (
        Option<Felt>,
        Option<Felt>,
        Option<Felt>,
        Option<Felt>,
        Option<Felt>,
        Option<Felt>,
    ) {
        let (next_ap, next_fp);
        let mut op0_update = None;
        let mut op1_update = None;
        let mut res_update = None;
        let mut dst_update = None;
        // The following branches don't include the assertions. That is done in the verification.
        if self.inst().opcode() == OPC_CALL {
            /*1*/
            // "call" instruction
            self.mem.write(self.curr.ap, self.curr.fp); // Save current fp
            self.mem
                .write(self.curr.ap + Felt::new(1), self.curr.pc + size); // Save next instruction

            dst_update = self.mem.read(self.curr.ap);
            op0_update = self.mem.read(self.curr.ap + Felt::new(1));

            // Update fp
            // pointer for next frame is after current fp and instruction after call
            next_fp = Some(self.curr.ap + Felt::from(2u32));

            // Update ap
            match self.inst().ap_up() {
                /*0*/
                AP_Z2 => next_ap = Some(self.curr.ap + Felt::from(2u32)), // two words were written so advance 2 positions
                _ => panic!("ap increment in call instruction"),
            };
        } else if self.inst().opcode() == OPC_JMP_INC /*0*/
            || self.inst().opcode() == OPC_RET /*2*/
            || self.inst().opcode() == OPC_AEQ
        /*4*/
        {
            // rest of types of instruction
            // jumps and increments || return || assert equal
            match self.inst().ap_up() {
                /*0*/ AP_Z2 => next_ap = Some(self.curr.ap), // no modification on ap
                /*1*/
                AP_ADD => {
                    // ap += <op> should be larger than current ap
                    next_ap = Some(self.curr.ap + res.expect("None res after AP_ADD"))
                }
                /*2*/ AP_ONE => next_ap = Some(self.curr.ap + Felt::new(1)), // ap++
                _ => panic!("Invalid ap_up flagset"),
            }

            match self.inst().opcode() {
                /*0*/
                OPC_JMP_INC => next_fp = Some(self.curr.fp), // no modification on fp
                /*2*/
                OPC_RET => next_fp = Some(dst.expect("None dst after OPC_RET")), // ret sets fp to previous fp that was in [ap-2]
                /*4*/
                OPC_AEQ => {
                    // The following conditional is a fix that is not explained in the whitepaper
                    // The goal is to distinguish two types of ASSERT_EQUAL where one checks that
                    // dst = res , but in order for this to be true, one sometimes needs to write
                    // the res in mem(dst_addr) and sometimes write dst in mem(res_dir). The only
                    // case where res can be None is when res = op1 and thus res_dir = op1_addr
                    if res.is_none() {
                        // res = dst
                        self.mem
                            .write(op1_addr, dst.expect("None dst after OPC_AEQ"));
                        op1_update = self.mem.read(op1_addr);
                        res_update = self.mem.read(op1_addr);
                    } else {
                        // dst = res
                        self.mem
                            .write(dst_addr, res.expect("None res after OPC_AEQ"));
                        dst_update = self.mem.read(dst_addr);
                    }
                    next_fp = Some(self.curr.fp); // no modification on fp
                }
                _ => {
                    panic!("This case must never happen")
                }
            }
        } else {
            panic!("Invalid opcode flagset");
        }
        (
            next_ap, next_fp, op0_update, op1_update, res_update, dst_update,
        )
    }
}

/// Trace-friendly state of the registers and instructions at
/// every step of program execution
pub struct State {
    registers: [Vec<Felt>; NUM_REGISTER_COLS],
    instructions: [Vec<Felt>; NUM_INSTRUCTION_COLS],
}

impl State {
    fn new(init_trace_len: usize) -> Self {
        let mut registers: Vec<Vec<Felt>> = Vec::with_capacity(NUM_REGISTER_COLS);
        let mut instructions: Vec<Vec<Felt>> = Vec::with_capacity(NUM_INSTRUCTION_COLS);
        for _ in 0..NUM_REGISTER_COLS {
            let column = Felt::zeroed_vector(init_trace_len);
            registers.push(column);
        }
        for _ in 0..NUM_INSTRUCTION_COLS {
            let column = Felt::zeroed_vector(init_trace_len);
            instructions.push(column);
        }
        State {
            registers: registers.try_into().unwrap(),
            instructions: instructions.try_into().unwrap(),
        }
    }

    fn set_register_state(&mut self, step: usize, s: RegisterState) {
        self.registers[0][step] = s.pc;
        self.registers[1][step] = s.ap;
        self.registers[2][step] = s.fp;
    }

    fn set_instruction_state(&mut self, step: usize, s: InstructionState) {
        // Flags
        let flags = s.inst.flags();
        for i in 0..=15 {
            self.instructions[i][step] = flags[i];
        }
        self.instructions[16][step] = s.inst.word();

        // Offsets
        self.instructions[17][step] = s.inst.off_dst();
        self.instructions[18][step] = s.inst.off_op0();
        self.instructions[19][step] = s.inst.off_op1();

        // Operands
        self.instructions[20][step] = s.dst_addr;
        self.instructions[21][step] = s.op0_addr;
        self.instructions[22][step] = s.op1_addr;

        // Auxiliary values
        self.instructions[23][step] = s.dst.unwrap_or(Felt::new(0));
        self.instructions[24][step] = s.op0.unwrap_or(Felt::new(0));
        self.instructions[25][step] = s.op1.unwrap_or(Felt::new(0));
        self.instructions[26][step] = s.res.unwrap_or(Felt::new(0));
    }

    pub fn into_trace(&self) -> [Vec<Felt>; STATE_TRACE_WIDTH] {
        let mut trace: Vec<Vec<Felt>> = Vec::with_capacity(STATE_TRACE_WIDTH);
        trace.extend_from_slice(&self.registers);
        trace.extend_from_slice(&self.instructions);
        let mut t0 = vec![];
        let mut t1 = vec![];
        for step in 0..self.registers[0].len() {
            t0.push(self.instructions[9][step] * self.instructions[21][step]);
            t1.push(t0[step] * self.instructions[24][step]);
        }
        trace.extend_from_slice(&[t0, t1]);
        trace.try_into().unwrap()
    }
}

/// This struct stores the needed information to run a program
pub struct Program<'a> {
    /// total number of steps
    steps: Felt,
    /// full execution memory
    mem: &'a mut Memory,
    /// initial register state
    init: RegisterState,
    /// final register state
    fin: RegisterState,
    /// hints
    hints: Option<HintManager>,
}

impl<'a> Program<'a> {
    /// Creates an execution from the public information (memory and initial pointers)
    pub fn new(mem: &mut Memory, pc: u64, ap: u64, hints: Option<HintManager>) -> Program {
        Program {
            steps: Felt::new(0),
            mem,
            init: RegisterState::new(Felt::from(pc), Felt::from(ap), Felt::from(ap)),
            fin: RegisterState::new(Felt::new(0), Felt::new(0), Felt::new(0)),
            hints,
        }
    }

    /// Outputs the total number of steps of the execution carried out by the runner
    pub fn get_steps(&self) -> Felt {
        self.steps
    }

    /// Outputs the final value of the pointers after the execution carried out by the runner
    pub fn get_final(&self) -> RegisterState {
        self.fin
    }

    /// This function simulates an execution of the program received as input
    /// and returns an execution trace
    pub fn execute(&mut self) -> Result<ExecutionTrace, ExecutionError> {
        let mut state = State::new(self.mem.size() as usize);
        let mut n: usize = 0;
        let mut end = false;
        let mut curr = self.init;
        let mut next = curr;

        // keep executing steps until the end is reached
        while !end {
            // create current step of computation
            let mut step = Step::new(self.mem, self.hints.as_ref(), next);
            curr = step.curr;

            // execute current step and save state
            let inst_state = step.execute();
            state.set_register_state(n, curr);
            state.set_instruction_state(n, inst_state);

            n += 1;
            match step.next {
                None => end = true,
                _ => {
                    next = step.next.expect("Empty next pointers");
                    if curr.ap.as_int() <= next.pc.as_int() {
                        // if reading from unallocated memory, end
                        end = true;
                    }
                }
            }
        }
        self.fin = curr;
        self.steps = Felt::from(n as u64);

        Ok(ExecutionTrace::new(n, &*self.mem, &state))
    }
}
