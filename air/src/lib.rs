use giza_core::{Felt, FieldElement};
use winter_air::{
    Air, AirContext, Assertion, EvaluationFrame, ProofOptions as WinterProofOptions, TraceInfo,
    TransitionConstraintDegree,
};
use winter_utils::{ByteWriter, Serializable};

// EXPORTS
// ================================================================================================

pub use winter_air::{FieldExtension, HashFunction};

mod options;
pub use options::ProofOptions;

mod constraints;
use constraints::EvaluationResult;

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

    fn new(trace_info: TraceInfo, pub_inputs: PublicInputs, options: WinterProofOptions) -> Self {
        let mut degrees = vec![];
        for _ in 0..=14 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        degrees.push(TransitionConstraintDegree::new(1));
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(1));
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(3));
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(2));

        Self {
            context: AirContext::new(trace_info, degrees, options),
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

    fn evaluate_transition<E: FieldElement + From<Felt>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        result.evaluate_instr_constraints(frame);
        result.evaluate_operand_constraints(frame);
        result.evaluate_register_constraints(frame);
        result.evaluate_opcode_constraints(frame);
        result.evaluate_memory_constraints(frame);
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
