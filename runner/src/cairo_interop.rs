/// Code for parsing the outputs of Starkware's cairo-runner.
/// Note the following:
/// - Field elements are encoded in little-endian byte order.
/// - Cairo serialises field elements as 32 bytes [1] in their file formats
///   (based on the size of the program prime).
///
use crate::memory::Memory;
use giza_core::{Felt, RegisterState};
use std::fs::{metadata, File};
use std::io::Read;
use std::path::PathBuf;

/// Parses an execution trace outputted by the cairo-runner.
/// e.g. cairo-runner --trace_file out/trace.bin
/// Note that the trace is not assumed to be padded to a power of 2.
pub fn read_trace_bin(path: PathBuf) -> Vec<RegisterState> {
    let mut f = File::open(&path).expect("no file found");
    let metadata = metadata(&path).expect("unable to read metadata");
    let length = metadata.len() as usize;

    // Buffer for register values
    let mut pc: [u8; 8] = Default::default();
    let mut ap: [u8; 8] = Default::default();
    let mut fp: [u8; 8] = Default::default();

    let mut ptrs: Vec<RegisterState> = vec![];
    let mut bytes_read = 0;
    while bytes_read < length {
        bytes_read += f.read(&mut ap).unwrap();
        bytes_read += f.read(&mut fp).unwrap();
        bytes_read += f.read(&mut pc).unwrap();
        let reg = RegisterState::new(
            u64::from_le_bytes(pc),
            u64::from_le_bytes(ap),
            u64::from_le_bytes(fp),
        );
        ptrs.push(reg);
    }

    return ptrs;
}

/// Parses a memory dump outputted by the cairo-runner.
/// e.g. cairo-runner --memory_file out/memory.bin
pub fn read_memory_bin(path: PathBuf) -> Memory {
    let mut f = File::open(&path).expect("no file found");
    let metadata = metadata(&path).expect("unable to read metadata");
    let length = metadata.len() as usize;

    // Buffer for memory accesses
    let mut address: [u8; 8] = Default::default();
    let mut value: [u8; 32] = Default::default();

    let mut public_mem = Memory::new(vec![]).clone();
    let mut bytes_read = 0;
    while bytes_read < length {
        bytes_read += f.read(&mut address).unwrap();
        bytes_read += f.read(&mut value).unwrap();
        public_mem.write(
            Felt::try_from(u64::from_le_bytes(address)).unwrap(),
            Felt::try_from(value).unwrap(),
        );
    }

    return public_mem;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_bin() {
        let trace = read_trace_bin(PathBuf::from("../tmp/trace.bin"));
        println!("{:?}", trace);
    }

    #[test]
    fn test_memory_bin() {
        let mem = read_memory_bin(PathBuf::from("../tmp/memory.bin"));
        println!("{:?}", mem.data);
    }
}
