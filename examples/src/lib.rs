use clap::{ArgEnum, Parser};
use runner::ExecutionTrace;

//use prover::StarkProof;
//use winterfell::VerifierError;

pub mod factorial;
pub mod fibonacci;
pub mod output;

#[derive(Parser)]
pub struct ExampleArgs {
    #[clap(value_enum)]
    pub example: ExampleType,

    #[clap(long)]
    pub prove: bool,
}

#[derive(clap::ValueEnum, Clone)]
pub enum ExampleType {
    Fibonacci,
    Factorial,
    Output,
}

//trait Example {
//    fn run() -> ExecutionTrace;
//}
