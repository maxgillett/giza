use air::{ProcessorAir, ProofOptions, PublicInputs};
use giza_core::Felt;
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
        //Felt::from(0x208b7fff7fff7ffeu64),
        Felt::from(0x10780017fff7fffu64), // infinite loop
    ];

    let mut mem = Memory::new(instrs);
    mem.write(Felt::from(21u32), Felt::from(41u32)); // beginning of output
    mem.write(Felt::from(22u32), Felt::from(44u32)); // end of output
    mem.write(Felt::from(23u32), Felt::from(44u32)); // end of program
    let mut program = Program::new(&mut mem, 5, 24);

    // execute the program and generate the proof of execution
    let proof_options = ProofOptions::with_96_bit_security();
    let (_outputs, proof) = prover::execute(&mut program, &proof_options).unwrap();

    // verify correct program execution
    let pc = vec![Felt::new(5), Felt::new(20)];
    let ap = vec![Felt::new(24), Felt::new(41)];
    let pub_inputs = PublicInputs::new(pc, ap);
    match winterfell::verify::<ProcessorAir>(proof, pub_inputs) {
        Ok(_) => println!("Execution verified"),
        Err(err) => println!("Failed to verify execution: {}", err),
    }
}
