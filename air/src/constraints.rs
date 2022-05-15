use super::{AuxEvaluationFrame, AuxTraceRandElements, MainEvaluationFrame};
use giza_core::{ExtensionOf, Felt, FieldElement, FlagDecomposition, OffsetDecomposition};

pub trait EvaluationResult<E: FieldElement> {
    fn evaluate_instr_constraints(&mut self, frame: &MainEvaluationFrame<E>);
    fn evaluate_operand_constraints(&mut self, frame: &MainEvaluationFrame<E>);
    fn evaluate_register_constraints(&mut self, frame: &MainEvaluationFrame<E>);
    fn evaluate_opcode_constraints(&mut self, frame: &MainEvaluationFrame<E>);
}

pub trait AuxEvaluationResult<E: FieldElement, F: FieldElement + ExtensionOf<E>> {
    fn evaluate_memory_constraints(
        &mut self,
        main_frame: &MainEvaluationFrame<E>,
        aux_frame: &AuxEvaluationFrame<F>,
        aux_rand_elements: &AuxTraceRandElements<F>,
    );
    fn evaluate_range_check_constraints(
        &mut self,
        main_frame: &MainEvaluationFrame<E>,
        aux_frame: &AuxEvaluationFrame<F>,
        aux_rand_elements: &AuxTraceRandElements<F>,
    );
}

/// Main constraint identifiers
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

const TWO: Felt = Felt::new(2);

impl<E: FieldElement + From<Felt>> EvaluationResult<E> for [E] {
    fn evaluate_instr_constraints(&mut self, frame: &MainEvaluationFrame<E>) {
        let curr = frame.current();
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

    fn evaluate_operand_constraints(&mut self, frame: &MainEvaluationFrame<E>) {
        let curr = frame.current();
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

    fn evaluate_register_constraints(&mut self, frame: &MainEvaluationFrame<E>) {
        let curr = frame.current();
        let next = frame.next();
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
        self[NEXT_PC_2] = E::from(0u8); // FIXME: Why is this constraint not evaluating to zero?
        self[T0] = curr.f_pc_jnz() * curr.dst() - curr.t0();
        self[T1] = curr.t0() * curr.res();
    }

    fn evaluate_opcode_constraints(&mut self, frame: &MainEvaluationFrame<E>) {
        let curr = frame.current();
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
}

impl<E, F> AuxEvaluationResult<E, F> for [F]
where
    E: FieldElement + From<Felt>,
    F: FieldElement + From<Felt> + ExtensionOf<E>,
{
    fn evaluate_memory_constraints(
        &mut self,
        main_frame: &MainEvaluationFrame<E>,
        aux_frame: &AuxEvaluationFrame<F>,
        aux_rand_elements: &AuxTraceRandElements<F>,
    ) {
        // TODO: Don't hardcode width and offset values

        let curr = main_frame.segment();
        let aux = aux_frame.segment();
        for i in 0..4 {
            let random_elements = aux_rand_elements.get_segment_elements(0);
            let z = random_elements[0];
            let alpha = random_elements[1];

            // Continuity constraint
            self[i] = (aux.a_m_prime(i + 1) - aux.a_m_prime(i))
                * (aux.a_m_prime(i + 1) - aux.a_m_prime(i) - F::ONE);

            // Single-valued constraint
            self[i + 4] = (aux.v_m_prime(i + 1) - aux.v_m_prime(i))
                * (aux.a_m_prime(i + 1) - aux.a_m_prime(i) - F::ONE);

            // Cumulative product step
            let a_m: F = curr.a_m(i).into();
            let v_m: F = curr.v_m(i).into();
            self[i + 8] = (z - (aux.a_m_prime(i) + alpha * aux.v_m_prime(i))) * aux.p_m(i + 1)
                - (z - (a_m + alpha * v_m)) * aux.p_m(i);
        }
    }

    fn evaluate_range_check_constraints(
        &mut self,
        main_frame: &MainEvaluationFrame<E>,
        aux_frame: &AuxEvaluationFrame<F>,
        aux_rand_elements: &AuxTraceRandElements<F>,
    ) {
        let curr = main_frame.segment();
        let aux = aux_frame.segment();
        for i in 0..4 {
            let random_elements = aux_rand_elements.get_segment_elements(1);
            let z = random_elements[0];

            // Continuity constraint
            self[i + 12] = (aux.a_rc_prime(i + 1) - aux.a_rc_prime(i))
                * (aux.a_rc_prime(i + 1) - aux.a_rc_prime(i) - F::ONE);

            // Cumulative product step
            self[i + 15] =
                (z - aux.a_rc_prime(i)) * aux.p_rc(i + 1) - (z - curr.a_rc(i).into()) * aux.p_rc(i)
        }
    }
}
