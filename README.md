## Overview

Giza leverages the Winterfell library to prove and verify the execution of programs running on the Cairo VM.

## Usage instructions

Giza offers two modes of usage. In the first mode, an execution trace created by an external Cairo runner is supplied to the CLI to output a proof. The provided trace consists of binary files containing the record of register and memory states visited during the run of a Cairo program. To prove execution, additional auxiliary trace values must be reconstructed, and the built-in Rust runner is used to re-execute the trace in order to compute these values.

The second usage mode accepts only a Cairo program and initial register state, and uses the runner to construct all necessary trace information (including trace and memory values). Unlike the first mode, Python hint support and program input are not yet fully supported. This is not the preferred mode of interacting with Giza, and is not currently exposed through the CLI.

### Mode 1: Supply a trace to the CLI

Assuming a compiled Cairo program `program.json`, the following steps can be taken to construct a proof:

1. Install the Giza CLI using nightly Rust: `cargo install --path cli`
2. Generate the partial trace using an external runner, for example: `cairo-run --program=program.json --layout=all --memory_file=memory.bin --trace_file=trace.bin`. Note that the Starkware runner may only be used for purposes that fall within its [license](https://github.com/starkware-libs/cairo-lang/blob/master/LICENSE.txt).
3. Construct the proof: `giza prove --trace=trace.bin --memory=memory.bin --program=program.json --output=output.bin`
4. Verify the proof: `giza verify --proof=output.bin`

### Mode 2: Supply a program

To prove and verify the execution of the program found in `examples/src/main.rs`, one can run the following after completing step 1 from the previous section.

`cargo run --release --bin giza-examples`

## Acknowledgments
- The Cairo virtual machine and programming language is developed by [Starkware](https://starkware.co/).
- The STARK prover and verifier is built using the [Winterfell](https://github.com/facebook/winterfell) project.
- The current Rust runner is a fork of the implementation written by Anaïs Querol of O(1) Labs.
