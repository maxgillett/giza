use crate::utils::Cmd;
use air::{ProcessorAir, ProofOptions};
use clap::{Parser, ValueHint};
use runner::ExecutionTrace;
use serde_json::Result;
//use std::io::prelude::*;
use std::path::PathBuf;

pub struct ProveOutput {}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ProveArgs {
    #[clap(
        help = "A path to the execution trace.",
        long,
        value_hint = ValueHint::FilePath
    )]
    pub trace: PathBuf,

    #[clap(
        help = "A path to the position-independent execution (PIE) file.",
        long,
        value_hint = ValueHint::FilePath
    )]
    pub memory: PathBuf,

    #[clap(
        help = "A path to write the STARK proof.",
        long,
        value_hint = ValueHint::FilePath
    )]
    pub output: PathBuf,
}

impl Cmd for ProveArgs {
    type Output = Result<ProveOutput>;

    fn run(self) -> Self::Output {
        // FIXME: When loading trace from file the codelen is not set in memory,
        // so public memory constraints are not applied

        // Load trace from file
        let trace = ExecutionTrace::from_file(self.trace, self.memory);

        // Generate proof
        let proof_options = ProofOptions::with_96_bit_security();
        let (proof, pub_inputs) = prover::prove_trace(trace, &proof_options).unwrap();
        let proof_bytes = proof.to_bytes();
        println!("Proof size: {:.1} KB", proof_bytes.len() as f64 / 1024f64);

        // TODO: Move this to a separate command (have two commands: prove, validate)
        // verify correct program execution
        match winterfell::verify::<ProcessorAir>(proof, pub_inputs) {
            Ok(_) => println!("Execution verified"),
            Err(err) => println!("Failed to verify execution: {}", err),
        }

        Ok(ProveOutput {})
    }
}
