use giza_core::Felt;
use runner::{ExecutionTrace, Memory, Program};

pub fn run() -> ExecutionTrace {
    let instrs: Vec<Felt> = vec![
        "0x480680017fff8000",
        "0x32",
        "0x1104800180018000",
        "0x03",
        "0x208b7fff7fff7ffe",
        "0x480680017fff8000",
        "0x01",
        "0x480680017fff8000",
        "0x01",
        "0x480680017fff8000",
        "0x03e8",
        "0x1104800180018000",
        "0x0d",
        "0x400680017fff7fff",
        "0x007de71c861c90f47f776d261de1ebe62e6887220d774b08eb7c9f66d2e888c2",
        "0x020780017fff7ffd",
        "0x04",
        "0x010780017fff7fff",
        "0x06",
        "0x482680017ffd8000",
        "0x0800000000000011000000000000000000000000000000000000000000000000",
        "0x1104800180018000",
        "0x0800000000000010fffffffffffffffffffffffffffffffffffffffffffffff1",
        "0x208b7fff7fff7ffe",
        "0x020780017fff7ffd",
        "0x04",
        "0x480a7ffc7fff8000",
        "0x208b7fff7fff7ffe",
        "0x480a7ffc7fff8000",
        "0x482a7ffc7ffb8000",
        "0x482680017ffd8000",
        "0x0800000000000011000000000000000000000000000000000000000000000000",
        "0x1104800180018000",
        "0x0800000000000010fffffffffffffffffffffffffffffffffffffffffffffff9",
        "0x208b7fff7fff7ffe",
    ]
    .iter()
    .map(|x| Felt::from_hex(&x[2..]))
    .collect::<Vec<_>>();

    let mut mem = Memory::new(instrs);
    let mut program = Program::new(&mut mem, 1, 38);
    let trace = program.execute().unwrap();
    trace
}
