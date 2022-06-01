### Overview
Giza leverages the Winterfell library to prove and verify the execution of programs running on the Cairo VM.

### Remaining tasks:
- [x] Implement the 251-bit STARK-friendly prime field chosen by Starkware
- [ ] Support zero knowledge (see relevant Winterfell [issue](https://github.com/novifinancial/winterfell/issues/9))
- [ ] Bitwise builtin constraints
- [ ] Python hint support
- [ ] Command line interface
    - [ ] Supply a Cairo program and its inputs, and output a proof
    - [ ] Supply a Giza-generated proof, and output whether verification was successful
- [ ] Compile to WASM and test in browser

### Running the example program
To verify the execution of the program found in `examples/src/main.rs`, you will need to do the following:
- Clone the branch of the Winterfell fork found [here](https://github.com/maxgillett/winterfell/tree/constraint_divisors) into the parent directory of this repository.
- Use nightly Rust
- Run `cargo run --release`
