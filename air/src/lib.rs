#![feature(generic_associated_types)]

use giza_core::{ExtensionOf, Felt, FieldElement, P_M_OFFSET};
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
    pc_init: Felt,
    ap_init: Felt,
    pc_fin: Felt,
    ap_fin: Felt,
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
        main_degrees.push(TransitionConstraintDegree::new(3)); // TODO: Add another trace column for MUL to reduce this to 2
        main_degrees.push(TransitionConstraintDegree::new(2));
        main_degrees.push(TransitionConstraintDegree::new(2));
        main_degrees.push(TransitionConstraintDegree::new(2));

        let aux_degrees = vec![
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
            pc_init: pub_inputs.pc_init,
            ap_init: pub_inputs.ap_init,
            pc_fin: pub_inputs.pc_fin,
            ap_fin: pub_inputs.ap_fin,
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Felt>> {
        let last_step = self.trace_length() - 1;
        vec![
            // pc assertions
            Assertion::single(0, 0, self.pc_init),
            Assertion::single(0, last_step, self.pc_fin),
            // ap assertions
            Assertion::single(1, 0, self.ap_init),
            Assertion::single(1, last_step, self.ap_fin),
        ]
    }

    fn get_aux_assertions<E: FieldElement + From<Self::BaseField>>(
        &self,
        aux_rand_elements: &AuxTraceRandElements<E>,
    ) -> Vec<Assertion<E>> {
        // TODO: Modify assertions to constrain public memory
        // TODO: Modify assertions to constrain rc_min and rc_max
        // TODO: Abstract away specific trace layout (i.e. P_M_OFFSET + 3)
        let last_step = self.trace_length() - 1;
        vec![Assertion::single(11, last_step, E::ONE)]
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
        //result.evaluate_range_check_constraints(main_frame, aux_frame, aux_rand_elements);
    }

    fn context(&self) -> &AirContext<Felt> {
        &self.context
    }
}

// PUBLIC INPUTS
// ================================================================================================

#[derive(Debug)]
pub struct PublicInputs {
    pc_init: Felt,
    ap_init: Felt,
    pc_fin: Felt,
    ap_fin: Felt,
}

impl PublicInputs {
    pub fn new(pc: Vec<Felt>, ap: Vec<Felt>) -> Self {
        Self {
            pc_init: pc[0],
            ap_init: ap[0],
            pc_fin: pc[1],
            ap_fin: ap[1],
        }
    }
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.ap_init);
        target.write(self.ap_init);
        target.write(self.pc_fin);
        target.write(self.ap_fin);
    }
}
