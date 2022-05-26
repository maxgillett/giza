use air::{ProcessorAir, ProofOptions, PublicInputs};
use giza_core::{Felt, RegisterState};
use runner::{Memory, Program};

fn main() {
    //  %builtins output
    //  from starkware.cairo.common.serialize import serialize_word
    //  func main{output_ptr : felt*}():
    //      tempvar x = 10
    //      tempvar y = x + x
    //      tempvar z = y * y + x
    //      serialize_word(x)
    //      serialize_word(y)
    //      serialize_word(z)
    //      __end__:
    //      jmp __end__
    //  end
    //  */
    let instrs: Vec<Felt> = vec![
        Felt::from(0x400380007ffc7ffdu64),
        Felt::from(0x482680017ffc8000u64),
        Felt::from(1u64),
        Felt::from(0x208b7fff7fff7ffeu64),
        Felt::from(0x480680017fff8000u64),
        Felt::from(10u64),
        Felt::from(0x48307fff7fff8000u64),
        Felt::from(0x48507fff7fff8000u64),
        Felt::from(0x48307ffd7fff8000u64),
        Felt::from(0x480a7ffd7fff8000u64),
        Felt::from(0x48127ffb7fff8000u64),
        Felt::from(0x1104800180018000u64),
        -Felt::from(11u64),
        Felt::from(0x48127ff87fff8000u64),
        Felt::from(0x1104800180018000u64),
        -Felt::from(14u64),
        Felt::from(0x48127ff67fff8000u64),
        Felt::from(0x1104800180018000u64),
        -Felt::from(17u64),
        Felt::from(0x208b7fff7fff7ffeu64),
    ];

    // Memory is 20 bytes.
    // Then we have
    // 41 # beginning of output
    // 44 # end of output
    // 44 # end of program
    // new Program(memory, address pointer, program counter, hints)
    // pc = 5
    // ap = 24
    // verify:
    // pc :  5 -> 20
    // ap : 24 -> 41
    let mut mem = Memory::new(instrs);
    mem.write_pub(Felt::from(21u32), Felt::from(41u32)); // beginning of output
    mem.write_pub(Felt::from(22u32), Felt::from(44u32)); // end of output
    mem.write_pub(Felt::from(23u32), Felt::from(44u32)); // end of program

    // run the program to create an execution trace
    let mut program = Program::new(&mut mem, 5, 24, None);
    let trace = program.execute().unwrap();

    // build the public inputs
    let num_steps = program.get_steps();
    let rc_min = trace.rc_min;
    let rc_max = trace.rc_max;
    let init = RegisterState::new(5u64, 24u64, 24u64);
    let fin = RegisterState::new(20u64, 41u64, 41u64);
    let pub_mem = trace.public_mem();
    let pub_inputs = PublicInputs::new(init, fin, rc_min, rc_max, pub_mem, num_steps);

    // generate the proof of execution
    let proof_options = ProofOptions::with_96_bit_security();
    let proof = prover::prove_trace(trace, &proof_options).unwrap();
    let proof_bytes = proof.to_bytes();
    println!("Proof size: {:.1} KB", proof_bytes.len() as f64 / 1024f64);

    // verify correct program execution
    match winterfell::verify::<ProcessorAir>(proof, pub_inputs) {
        Ok(_) => println!("Execution verified"),
        Err(err) => println!("Failed to verify execution: {}", err),
    }
}
