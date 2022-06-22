use giza_core::Felt;
use runner::{ExecutionTrace, Memory, Program};

pub fn run() -> ExecutionTrace {
    let instrs: Vec<Felt> = vec![
        "0x480680017fff8000",
        "0x32",
        "0x1104800180018000",
        "0x3",
        "0x208b7fff7fff7ffe",
        "0x480680017fff8000",
        "0x1",
        "0x480680017fff8000",
        "0x1",
        "0x480680017fff8000",
        "0x3e8",
        "0x1104800180018000",
        "0xd",
        "0x400680017fff7fff",
        "0x7de71c861c90f47f776d261de1ebe62e6887220d774b08eb7c9f66d2e888c2",
        "0x20780017fff7ffd",
        "0x4",
        "0x10780017fff7fff",
        "0x6",
        "0x482680017ffd8000",
        "0x800000000000011000000000000000000000000000000000000000000000000",
        "0x1104800180018000",
        "0x800000000000010fffffffffffffffffffffffffffffffffffffffffffffff1",
        "0x208b7fff7fff7ffe",
        "0x20780017fff7ffd",
        "0x4",
        "0x480a7ffc7fff8000",
        "0x208b7fff7fff7ffe",
        "0x480a7ffc7fff8000",
        "0x482a7ffc7ffb8000",
        "0x482680017ffd8000",
        "0x800000000000011000000000000000000000000000000000000000000000000",
        "0x1104800180018000",
        "0x800000000000010fffffffffffffffffffffffffffffffffffffffffffffff9",
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
