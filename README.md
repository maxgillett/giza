### Overview
Giza leverages the Winterfell library to prove and verify the execution of programs running on the Cairo VM.

### Remaining tasks:
- [ ] Implement the 251-bit STARK-friendly prime field chosen by Starkware
- [ ] Support zero knowledge (see relevant Winterfell [issue](https://github.com/novifinancial/winterfell/issues/9))
- [ ] Bitwise builtin constraints
- [ ] Python hint support
- [ ] Command line interface
    - [ ] Supply a Cairo program and its inputs, and output a proof
    - [ ] Supply a Giza-generated proof, and output whether verification was successful
- [ ] Compile to WASM and test in browser
