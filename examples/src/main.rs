use air::{ProcessorAir, ProofOptions, PublicInputs};
use giza_core::{Felt, RegisterState};
//use runner::hints::{Hint, HintManager};
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
    mem.write_pub(Felt::from(21u32), Felt::from(41u32)); // beginning of output
    mem.write_pub(Felt::from(22u32), Felt::from(44u32)); // end of output
    mem.write_pub(Felt::from(23u32), Felt::from(44u32)); // end of program

    //let mut hints = HintManager::default();
    //hints.push_hint(7, Hint::new(String::from("memory[30]=5"), vec![], None));
    //hints.push_hint(7, Hint::new(String::from("memory[31]=5"), vec![], None));

    let mut program = Program::new(&mut mem, 5, 24, None);
    let num_steps = program.get_steps();

    // execute the program and generate the proof of execution
    let proof_options = ProofOptions::with_96_bit_security();
    let (_outputs, proof) = prover::execute(&mut program, &proof_options).unwrap();
    let proof_bytes = proof.to_bytes();
    println!("Proof size: {:.1} KB", proof_bytes.len() as f64 / 1024f64);

    // verify correct program execution
    let init = RegisterState::new(5u64, 24u64, 24u64);
    let fin = RegisterState::new(20u64, 41u64, 41u64);
    let rc_min = 0;
    let rc_max = 200;
    let pub_inputs = PublicInputs::new(init, fin, rc_min, rc_max, mem.data, num_steps);
    match winterfell::verify::<ProcessorAir>(proof, pub_inputs) {
        Ok(_) => println!("Execution verified"),
        Err(err) => println!("Failed to verify execution: {}", err),
    }
}
