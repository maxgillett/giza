use giza_core::Felt;
use runner::{ExecutionTrace, Memory, Program};

pub fn run() -> ExecutionTrace {
    let instrs: Vec<Felt> = vec![
        "0x482680017ffd8000",
        "0x800000000000011000000000000000000000000000000000000000000000000",
        "0x20680017fff7fff",
        "0x4",
        "0x480a7ffd7fff8000",
        "0x208b7fff7fff7ffe",
        "0x482680017ffd8000",
        "0x800000000000011000000000000000000000000000000000000000000000000",
        "0x1104800180018000",
        "0x800000000000010fffffffffffffffffffffffffffffffffffffffffffffff9",
        "0x48527fff7ffd8000",
        "0x208b7fff7fff7ffe",
        "0x480a7ffc7fff8000",
        "0x1104800180018000",
        "0x800000000000010fffffffffffffffffffffffffffffffffffffffffffffff4",
        "0x20780017fff7ffd",
        "0x4",
        "0x10780017fff7fff",
        "0x7",
        "0x480a7ffc7fff8000",
        "0x482680017ffd8000",
        "0x800000000000011000000000000000000000000000000000000000000000000",
        "0x1104800180018000",
        "0x800000000000010fffffffffffffffffffffffffffffffffffffffffffffff7",
        "0x208b7fff7fff7ffe",
        "0x480680017fff8000",
        "0xa",
        "0x1104800180018000",
        "0x800000000000010ffffffffffffffffffffffffffffffffffffffffffffffe6",
        "0x400680017fff7fff",
        "0x375f00",
        "0x480680017fff8000",
        "0x2710",
        "0x480680017fff8000",
        "0xa",
        "0x1104800180018000",
        "0x800000000000010ffffffffffffffffffffffffffffffffffffffffffffffea",
        "0x208b7fff7fff7ffe",
    ]
    .iter()
    .map(|x| Felt::from(x))
    .collect::<Vec<_>>();

    let mut mem = Memory::new(instrs);
    mem.write_pub(Felt::from(21u32), Felt::from(41u32)); // beginning of output
    mem.write_pub(Felt::from(22u32), Felt::from(44u32)); // end of output
    mem.write_pub(Felt::from(23u32), Felt::from(44u32)); // end of program

    let mut program = Program::new(&mut mem, 5, 24);
    let trace = program.execute().unwrap();
    trace
}
