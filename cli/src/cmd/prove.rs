use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use super::ProofData;
use crate::utils::Cmd;
use air::ProofOptions;
use clap::{Parser, ValueHint};
use runner::ExecutionTrace;
use winter_utils::Serializable;

pub struct ProveOutput {}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ProveArgs {
    #[clap(
        help = "Path to the compiled Cairo program JSON file",
        long,
        value_hint = ValueHint::FilePath
    )]
    pub program: PathBuf,

    #[clap(
        help = "Path to the execution trace output file",
        long,
        value_hint = ValueHint::FilePath
    )]
    pub trace: PathBuf,

    #[clap(
        help = "Path to the memory output file",
        long,
        value_hint = ValueHint::FilePath
    )]
    pub memory: PathBuf,

    #[clap(
        help = "Path to write the STARK proof",
        long,
        value_hint = ValueHint::FilePath
    )]
    pub output: PathBuf,
}

#[derive(Debug)]
pub enum Error {}

impl Cmd for ProveArgs {
    type Output = Result<ProveOutput, Error>;

    fn run(self) -> Self::Output {
        // Load trace from file
        let trace = ExecutionTrace::from_file(self.program, self.trace, self.memory);

        // Generate proof
        let proof_options = ProofOptions::with_96_bit_security();
        let (proof, pub_inputs) = prover::prove_trace(trace, &proof_options).unwrap();
        let input_bytes = pub_inputs.to_bytes();
        let proof_bytes = proof.to_bytes();
        println!("Proof size: {:.1} KB", proof_bytes.len() as f64 / 1024f64);

        // Write proof to disk
        let data = ProofData {
            input_bytes,
            proof_bytes,
        };
        let b = bincode::serialize(&data).unwrap();
        let mut f = File::create(self.output).unwrap();
        f.write_all(&b).unwrap();

        Ok(ProveOutput {})
    }
}
