//use crate::Example;
//use air::{ProcessorAir, ProofOptions};
use giza_core::Felt;
use runner::{ExecutionTrace, Memory, Program};

pub fn run() -> ExecutionTrace {
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
    let mut mem = Memory::new(instrs);
    mem.write_pub(Felt::from(21u32), Felt::from(41u32)); // beginning of output
    mem.write_pub(Felt::from(22u32), Felt::from(44u32)); // end of output
    mem.write_pub(Felt::from(23u32), Felt::from(44u32)); // end of program

    let mut program = Program::new(&mut mem, 5, 24);
    let trace = program.execute().unwrap();
    trace
}
