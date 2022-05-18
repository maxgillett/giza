use crate::{
    utils::{Cmd},
};
use clap::{Parser, ValueHint};
use std::{path::PathBuf};
use air::{ProofOptions};
use runner::{
    ExecutionTrace
};

// Arguments.
// 

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
        help = "A path to write the STARK proof.",
        long,
        value_hint = ValueHint::FilePath
    )]
    pub output: PathBuf,
}

pub struct ProveOutput {}

// Command.
// 

impl Cmd for ProveArgs {
    type Output = ProveOutput;

    fn run(self) -> Self::Output {
        // Read trace from trace file.
        // Verify it is generated by Giza?
        
        let proof_options = ProofOptions::with_96_bit_security();
        // TODO.
        let trace = runner::load_trace_from_file("");

        let proof = prover::prove(trace, &proof_options).unwrap();
        let proof_bytes = proof.to_bytes();
        println!("Proof size: {:.1} KB", proof_bytes.len() as f64 / 1024f64);
        
        ProveOutput {}
    }
}