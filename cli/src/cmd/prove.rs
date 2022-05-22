use crate::{
    utils::{Cmd},
};
use clap::{Parser, ValueHint};
use std::{path::PathBuf};
use air::{ProofOptions};
use runner::{
    ExecutionTrace
};
use std::io::prelude::*;
use std::fs::File;
use std::error::Error;

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

    // #[clap(
    //     help = "A path to the position-independent execution (PIE) file.",
    //     long,
    //     value_hint = ValueHint::FilePath
    // )]
    // pub pie: PathBuf,

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

pub struct ProveOutput {}

// PIE file.
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
struct ExecutionResources {
    n_steps: u32,
}



// Command.
// 

impl Cmd for ProveArgs {
    type Output = Result<ProveOutput>;

    fn run(self) -> Self::Output {
        // Read the PIE metadata.
        // let mut pie_file = File::open(self.pie)?;
        // let mut zip = zip::ZipArchive::new(&pie_file)?;
        // {
        //     let execution_resources = zip.by_name("execution_resources.json").unwrap().read();
        //     let memory_data = zip.by_name("memory.bin")?;
        // }

        /// Calculate the pc start/end
        // ```
        // compiled_json = read(pie, 'compiled.json')
        // compiled_json[identifiers.__main__.__start__/__end__]
        // start_pc = program_base + __start__
        // end_pc = program_base + __end__
        // ```

        /// Calculate the ap start/end
        // ```
        // # execution_base points towards execution segment start index.
        // # load metadata.json, load all segments by their index. 
        // # then compute execution_segment.base from the reduced sizes of all preceding segments.
        // start_ap = execution_base + 2
        // end_ap = ???
        // ```

        let proof_options = ProofOptions::with_96_bit_security();
        // TODO.
        let trace = runner::load_trace_from_file(
            self.trace,
            self.memory
        );

        let proof = prover::prove(trace, &proof_options).unwrap();
        let proof_bytes = proof.to_bytes();
        println!("Proof size: {:.1} KB", proof_bytes.len() as f64 / 1024f64);
        
        Ok(ProveOutput {})
    }
}