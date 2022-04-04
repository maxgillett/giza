use super::EvaluationFrame;
use giza_core::{flags::*, Decomposition, Felt, FieldElement};

pub trait EvaluationResult<E: FieldElement + From<Felt>> {
    fn evaluate_instr_constraints(&mut self, frame: &EvaluationFrame<E>);
    fn evaluate_operand_constraints(&mut self, frame: &EvaluationFrame<E>);
    fn evaluate_register_constraints(&mut self, frame: &EvaluationFrame<E>);
    fn evaluate_opcode_constraints(&mut self, frame: &EvaluationFrame<E>);
    fn evaluate_memory_constraints(&mut self, frame: &EvaluationFrame<E>);
}

const TWO: Felt = Felt::new(2);

/// Constraint numbers for evaluation result
pub const INST: usize = 16;
pub const DST_ADDR: usize = 17;
pub const OP0_ADDR: usize = 18;
pub const OP1_ADDR: usize = 19;
pub const NEXT_AP: usize = 20;
pub const NEXT_FP: usize = 21;
pub const NEXT_PC_1: usize = 22;
pub const NEXT_PC_2: usize = 23;
pub const T0: usize = 24;
pub const T1: usize = 25;
pub const MUL: usize = 26;
pub const CALL_1: usize = 27;
pub const CALL_2: usize = 28;
pub const ASSERT_EQ: usize = 29;
pub const MEMORY_1: usize = 30;
pub const MEMORY_2: usize = 31;

impl<E: FieldElement + From<Felt>> EvaluationResult<E> for [E] {
    fn evaluate_instr_constraints(&mut self, frame: &EvaluationFrame<E>) {
        let curr = TraceRow::new(frame.current());
        // Bit constraints
        for (n, flag) in curr.flags().into_iter().enumerate() {
            self[n] = match n {
                0..=14 => flag * (flag - Felt::ONE.into()),
                15 => flag,
                _ => panic!("Unknown flag offset"),
            };
        }
        // Instruction unpacking
        let b15: E = TWO.exp(15).into();
        let b16: E = TWO.exp(16).into();
        let b32: E = TWO.exp(32).into();
        let b48: E = TWO.exp(48).into();
        let a: E = curr
            .flags()
            .into_iter()
            .enumerate()
            .take(15)
            .fold(Felt::ZERO.into(), |acc, (n, flag)| {
                acc + E::from(2u128.pow(n as u32)) * flag
            });
        self[INST] = (curr.off_dst() + b15)
            + b16 * (curr.off_op0() + b15)
            + b32 * (curr.off_op1() + b15)
            + b48 * a
            - curr.inst();
    }

    fn evaluate_operand_constraints(&mut self, frame: &EvaluationFrame<E>) {
        let curr = TraceRow::new(frame.current());
        let ap = curr.ap();
        let fp = curr.fp();
        let pc = curr.pc();
        let one: E = Felt::ONE.into();
        self[DST_ADDR] =
            curr.f_dst_fp() * fp + (one - curr.f_dst_fp()) * ap + curr.off_dst() - curr.dst_addr();
        self[OP0_ADDR] =
            curr.f_op0_fp() * fp + (one - curr.f_op0_fp()) * ap + curr.off_op0() - curr.op0_addr();
        self[OP1_ADDR] = curr.f_op1_val() * pc
            + curr.f_op1_ap() * ap
            + curr.f_op1_fp() * fp
            + (one - curr.f_op1_val() - curr.f_op1_ap() - curr.f_op1_fp()) * curr.op0()
            + curr.off_op1()
            - curr.op1_addr();
    }

    fn evaluate_register_constraints(&mut self, frame: &EvaluationFrame<E>) {
        let curr = TraceRow::new(frame.current());
        let next = TraceRow::new(frame.next());
        let one: E = Felt::ONE.into();

        // ap and fp constraints
        self[NEXT_AP] = curr.ap()
            + curr.f_ap_add() * curr.res()
            + curr.f_ap_one()
            + curr.f_opc_call() * TWO.into()
            - next.ap();
        self[NEXT_FP] = curr.f_opc_ret() * curr.dst()
            + curr.f_opc_call() * (curr.ap() + TWO.into())
            + (one - curr.f_opc_ret() - curr.f_opc_call()) * curr.fp()
            - next.fp();

        // pc constraints
        self[NEXT_PC_1] =
            (curr.t1() - curr.f_pc_jnz()) * (next.pc() - (curr.pc() + curr.inst_size()));
        self[NEXT_PC_2] = curr.t0() * (next.pc() - (curr.pc() + curr.op1()))
            + (one - curr.f_pc_jnz()) * next.pc()
            - (one - curr.f_pc_abs() - curr.f_pc_rel() - curr.f_pc_jnz())
                * (curr.pc() + curr.inst_size())
            + curr.f_pc_abs() * curr.res()
            + curr.f_pc_rel() * (curr.pc() + curr.res());
        self[NEXT_PC_2] = E::from(0u8); // FIXME
        self[T0] = curr.f_pc_jnz() * curr.dst() - curr.t0();
        self[T1] = curr.t0() * curr.res();
    }

    fn evaluate_opcode_constraints(&mut self, frame: &EvaluationFrame<E>) {
        let curr = TraceRow::new(frame.current());
        let one: E = Felt::ONE.into();
        let mul = curr.op0() * curr.op1();

        self[MUL] = curr.f_res_add() * (curr.op0() + curr.op1())
            + curr.f_res_mul() * mul
            + (one - curr.f_res_add() - curr.f_res_mul() - curr.f_pc_jnz()) * curr.op1()
            - (one - curr.f_pc_jnz()) * curr.res();
        self[CALL_1] = curr.f_opc_call() * (curr.dst() - curr.fp());
        self[CALL_2] = curr.f_opc_call() * (curr.op0() - (curr.pc() + curr.inst_size()));
        self[ASSERT_EQ] = curr.f_opc_aeq() * (curr.dst() - curr.res());
    }

    fn evaluate_memory_constraints(&mut self, _frame: &EvaluationFrame<E>) {
        // TODO
        //let curr = TraceRow::new(frame.current());
        //let next = TraceRow::new(frame.next());
        //// Continuity constraint
        //self[MEMORY_1] =
        //    (next.a_prime() - curr.a_prime()) * (next.a_prime() - curr.a_prime() - ONE.into());
        //// Single-valued constraint
        //self[MEMORY_2] =
        //    (next.v_prime() - curr.v_prime()) * (next.a_prime() - curr.a_prime() - ONE.into());
        //// Permutation constraints
    }
}

struct TraceRow<'a, E: FieldElement + From<Felt>>(&'a [E]);

impl<'a, E: FieldElement + From<Felt>> TraceRow<'a, E> {
    pub fn new(v: &'a [E]) -> TraceRow<'a, E> {
        TraceRow(v)
    }
    /// Registers
    fn pc(&self) -> E {
        self.0[0]
    }
    fn ap(&self) -> E {
        self.0[1]
    }
    fn fp(&self) -> E {
        self.0[2]
    }
    /// Operand offsets
    fn inst(&self) -> E {
        self.0[19]
    }
    fn inst_size(&self) -> E {
        self.f_op1_val() + Felt::ONE.into()
    }
    fn dst_addr(&self) -> E {
        // TODO
        self.0[23]
    }
    fn op0_addr(&self) -> E {
        // TODO
        self.0[24]
    }
    fn op1_addr(&self) -> E {
        // TODO
        self.0[25]
    }
    /// Auxiliary values
    fn dst(&self) -> E {
        self.0[26]
    }
    fn op0(&self) -> E {
        self.0[27]
    }
    fn op1(&self) -> E {
        self.0[28]
    }
    fn res(&self) -> E {
        self.0[29]
    }
    /// Constraint auxiliary values
    fn t0(&self) -> E {
        self.0[30]
    }
    fn t1(&self) -> E {
        self.0[31]
    }
    ///// Memory
    //fn a_prime(&self) -> E {
    //    self.0[29]
    //}
    //fn v_prime(&self) -> E {
    //    self.0[30]
    //}
}

impl<'a, E: FieldElement + From<Felt>> Decomposition<E> for TraceRow<'a, E> {
    fn flags(&self) -> Vec<E> {
        let mut flags = Vec::with_capacity(NUM_FLAGS);
        // The most significant 16 bits
        for i in 0..NUM_FLAGS {
            flags.push(self.flag_at(i));
        }
        flags
    }

    fn flag_at(&self, pos: usize) -> E {
        self.0[3 + pos] //  - E::from(2u8) * self.0[3 + pos + 1]
    }

    fn f_dst_fp(&self) -> E {
        self.0[3]
    }

    fn f_op0_fp(&self) -> E {
        self.0[4]
    }

    fn f_op1_val(&self) -> E {
        self.0[5]
    }

    fn f_op1_fp(&self) -> E {
        self.0[6]
    }

    fn f_op1_ap(&self) -> E {
        self.0[7]
    }

    fn f_res_add(&self) -> E {
        self.0[8]
    }

    fn f_res_mul(&self) -> E {
        self.0[9]
    }

    fn f_pc_abs(&self) -> E {
        self.0[10]
    }

    fn f_pc_rel(&self) -> E {
        self.0[11]
    }

    fn f_pc_jnz(&self) -> E {
        self.0[12]
    }

    fn f_ap_add(&self) -> E {
        self.0[13]
    }

    fn f_ap_one(&self) -> E {
        self.0[14]
    }

    fn f_opc_call(&self) -> E {
        self.0[15]
    }

    fn f_opc_ret(&self) -> E {
        self.0[16]
    }

    fn f_opc_aeq(&self) -> E {
        self.0[17]
    }

    fn f15(&self) -> E {
        self.0[18]
    }

    fn off_dst(&self) -> E {
        self.0[20]
    }

    fn off_op0(&self) -> E {
        self.0[21]
    }

    fn off_op1(&self) -> E {
        self.0[22]
    }
}
