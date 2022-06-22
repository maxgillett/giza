use air::{ProcessorAir, ProofOptions};
use clap::Parser;
use examples::{factorial, fibonacci, output, ExampleArgs, ExampleType};

fn main() {
    let args = ExampleArgs::parse();

    let trace = match args.example {
        ExampleType::Fibonacci => fibonacci::run(),
        ExampleType::Factorial => factorial::run(),
        ExampleType::Output => output::run(),
    };

    if args.prove {
        // generate the proof of execution
        let proof_options = ProofOptions::with_96_bit_security();
        let (proof, pub_inputs) = prover::prove_trace(trace, &proof_options).unwrap();
        let proof_bytes = proof.to_bytes();
        println!("Proof size: {:.1} KB", proof_bytes.len() as f64 / 1024f64);

        // verify correct program execution
        match winterfell::verify::<ProcessorAir>(proof, pub_inputs) {
            Ok(_) => println!("Execution verified"),
            Err(err) => println!("Failed to verify execution: {}", err),
        }
    }
}
