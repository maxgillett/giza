use air::{ProcessorAir, PublicInputs};
use giza_core::{Felt, RegisterState, MEM_A_TRACE_OFFSET, MEM_P_TRACE_OFFSET};
use prover::{Prover, Trace};
use runner::{ExecutionError, ExecutionTrace, Program};

// EXPORTS
// ================================================================================================

pub use air::{FieldExtension, HashFunction, ProofOptions};
pub use prover::StarkProof;

// EXECUTOR
// ================================================================================================

/// Executes the specified `program` and returns the result together with a STARK-based proof of execution.
pub fn execute(
    program: &mut Program,
    options: &ProofOptions,
) -> Result<(Vec<u64>, StarkProof), ExecutionError> {
    // execute the program to create an execution trace
    let trace = program.execute()?;
    let outputs = vec![];

    // generate STARK proof
    let prover = ExecutionProver::new(options.clone());
    let proof = prover.prove(trace).map_err(ExecutionError::ProverError)?;

    Ok((outputs, proof))
}

// PROVER
// ================================================================================================

struct ExecutionProver {
    options: ProofOptions,
}

impl ExecutionProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for ExecutionProver {
    type BaseField = Felt;
    type Air = ProcessorAir;
    type Trace = ExecutionTrace;

    fn options(&self) -> &prover::ProofOptions {
        &self.options
    }

    fn get_pub_inputs(&self, trace: &ExecutionTrace) -> PublicInputs {
        let last_step = trace.length() - 1;
        let pc_init = trace.main_segment().get(MEM_A_TRACE_OFFSET, 0);
        let ap_init = trace.main_segment().get(MEM_P_TRACE_OFFSET, 0);
        let init = RegisterState::new(pc_init, ap_init, ap_init);

        let pc_fin = trace.main_segment().get(MEM_A_TRACE_OFFSET, last_step);
        let ap_fin = trace.main_segment().get(MEM_P_TRACE_OFFSET, last_step);
        let fin = RegisterState::new(pc_fin, ap_fin, ap_fin);

        let rc_min = trace.rc_min;
        let rc_max = trace.rc_max;

        let mem = trace.public_mem();

        PublicInputs::new(init, fin, rc_min, rc_max, mem, trace.num_steps)
    }
}
