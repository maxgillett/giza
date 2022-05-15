#![feature(generic_associated_types)]

use giza_core::{
    range, ExtensionOf, Felt, FieldElement, RegisterState, Word, A_RC_PRIME_OFFSET,
    MEM_A_TRACE_OFFSET, MEM_P_TRACE_OFFSET, P_M_OFFSET,
};

use winter_air::{
    Air, AirContext, Assertion, AuxTraceRandElements, ProofOptions as WinterProofOptions,
    TraceInfo, TransitionConstraintDegree,
};
use winter_utils::{ByteWriter, Serializable};

// EXPORTS
// ================================================================================================

pub use winter_air::{EvaluationFrame, FieldExtension, HashFunction};

mod options;
pub use options::ProofOptions;

mod constraints;
use constraints::{AuxEvaluationResult, EvaluationResult};

mod frame;
pub use frame::{AuxEvaluationFrame, MainEvaluationFrame};

// PROCESSOR AIR
// ================================================================================================

/// TODO: add docs
pub struct ProcessorAir {
    context: AirContext<Felt>,
    pub_inputs: PublicInputs,
}

impl Air for ProcessorAir {
    type BaseField = Felt;
    type PublicInputs = PublicInputs;
    type Frame<E: FieldElement> = MainEvaluationFrame<E>;
    type AuxFrame<E: FieldElement> = AuxEvaluationFrame<E>;

    fn new(trace_info: TraceInfo, pub_inputs: PublicInputs, options: WinterProofOptions) -> Self {
        let mut main_degrees = vec![];
        for _ in 0..=14 {
            main_degrees.push(TransitionConstraintDegree::new(2));
        }
        main_degrees.push(TransitionConstraintDegree::new(1));
        main_degrees.push(TransitionConstraintDegree::new(2));
        main_degrees.push(TransitionConstraintDegree::new(2));
        main_degrees.push(TransitionConstraintDegree::new(2));
        main_degrees.push(TransitionConstraintDegree::new(2));
        main_degrees.push(TransitionConstraintDegree::new(1));
        main_degrees.push(TransitionConstraintDegree::new(2));
        main_degrees.push(TransitionConstraintDegree::new(2));
        main_degrees.push(TransitionConstraintDegree::new(2));
        main_degrees.push(TransitionConstraintDegree::new(2));
        main_degrees.push(TransitionConstraintDegree::new(2));
        // TODO: Add another trace column for MUL to reduce this degree to 2
        main_degrees.push(TransitionConstraintDegree::new(3));
        main_degrees.push(TransitionConstraintDegree::new(2));
        main_degrees.push(TransitionConstraintDegree::new(2));
        main_degrees.push(TransitionConstraintDegree::new(2));

        let aux_degrees = vec![
            // Memory constraints
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            // Range check constraints
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
        ];

        Self {
            context: AirContext::new_multi_segment(
                trace_info,
                main_degrees,
                aux_degrees,
                4,
                1,
                options,
            ),
            pub_inputs,
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Felt>> {
        let last_step = self.trace_length() - 1;
        vec![
            // pc assertions
            Assertion::single(MEM_A_TRACE_OFFSET, 0, self.pub_inputs.init.pc),
            Assertion::single(MEM_A_TRACE_OFFSET, last_step, self.pub_inputs.fin.pc),
            // ap assertions
            Assertion::single(MEM_P_TRACE_OFFSET, 0, self.pub_inputs.init.ap),
            Assertion::single(MEM_P_TRACE_OFFSET, last_step, self.pub_inputs.fin.ap),
        ]
    }

    // TODO: Abstract away specific trace layout (i.e. P_M_OFFSET + 3)
    fn get_aux_assertions<E: FieldElement + From<Self::BaseField>>(
        &self,
        aux_rand_elements: &AuxTraceRandElements<E>,
    ) -> Vec<Assertion<E>> {
        // Constrain the following:
        // - Public memory
        // - Minimum range check value
        // - Maximum range check value

        let last_step = self.trace_length() - 1;
        let random_elements = aux_rand_elements.get_segment_elements(0);
        let z = random_elements[0];
        let alpha = random_elements[1];
        let num = z.exp((self.pub_inputs.mem.len() as u64).into());
        let den = self
            .pub_inputs
            .mem
            .iter()
            .enumerate()
            .map(|(a, v)| z - (E::from(a as u64) + alpha * E::from(v.unwrap().word())))
            .reduce(|a, b| a * b)
            .unwrap();

        vec![
            Assertion::single(P_M_OFFSET + 3, last_step, num / den),
            Assertion::single(A_RC_PRIME_OFFSET, 0, E::from(self.pub_inputs.rc_min)),
            Assertion::single(
                A_RC_PRIME_OFFSET + 2,
                last_step,
                E::from(self.pub_inputs.rc_max),
            ),
        ]
    }

    fn evaluate_transition<E: FieldElement + From<Felt>>(
        &self,
        frame: &MainEvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        result.evaluate_instr_constraints(frame);
        result.evaluate_operand_constraints(frame);
        result.evaluate_register_constraints(frame);
        result.evaluate_opcode_constraints(frame);
    }

    fn evaluate_aux_transition<
        E: FieldElement + From<Felt>,
        F: FieldElement + From<Felt> + ExtensionOf<E>,
    >(
        &self,
        main_frame: &MainEvaluationFrame<E>,
        aux_frame: &AuxEvaluationFrame<F>,
        _periodic_values: &[E],
        aux_rand_elements: &AuxTraceRandElements<F>,
        result: &mut [F],
    ) {
        result.evaluate_memory_constraints(main_frame, aux_frame, aux_rand_elements);
        result.evaluate_range_check_constraints(main_frame, aux_frame, aux_rand_elements);
    }

    fn context(&self) -> &AirContext<Felt> {
        &self.context
    }
}

// PUBLIC INPUTS
// ================================================================================================

#[derive(Debug)]
pub struct PublicInputs {
    init: RegisterState,    // initial register state
    fin: RegisterState,     // final register state
    rc_min: u16,            // minimum range check value (0 < rc_min < rc_max < 2^16)
    rc_max: u16,            // maximum range check value
    mem: Vec<Option<Word>>, // public memory
}

impl PublicInputs {
    pub fn new(
        init: RegisterState,
        fin: RegisterState,
        rc_min: u16,
        rc_max: u16,
        mem: Vec<Option<Word>>,
    ) -> Self {
        Self {
            init,
            fin,
            rc_min,
            rc_max,
            mem,
        }
    }
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.init.pc);
        target.write(self.init.ap);
        target.write(self.fin.pc);
        target.write(self.fin.ap);
    }
}
