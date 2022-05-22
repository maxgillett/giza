// Code for parsing the outputs of Starkware's cairo-runner.
use std::path::PathBuf;
use crate::memory::Memory;
use giza_core::{
    Felt
};
use std::fs::File;
use std::fs;
use std::io::Read;
use std::iter;
// use winterfell::{Matrix, Trace, TraceLayout};


// Notes on Cairo's file fornats.
// 
// Field elements are encoded in little-endian byte order.
// 
// Cairo serialises field elements as 32 bytes [1] in their file formats.
// This is based on the size of the program prime. 
// 
// [1]:
// ```py
// # Based on https://sourcegraph.com/github.com/starkware-libs/cairo-lang@2abd303e1808612b724bc1412b2b5babd04bb4e7/-/blob/src/starkware/cairo/lang/vm/cairo_run.py?L368:9
// program.prime = 3618502788666131213697322783095070105623107215331596699973092056135872020481
// field_bytes = math.ceil(program.prime.bit_length() / 8)
// ```

struct MemoryItem {
    // little endian
    address: [u8; 8],
    value: [u8; 32],
}

struct TraceItem {
    ap: [u8; 8],
    fp: [u8; 8],
    pc: [u8; 8]
}


/// Parses an execution trace outputted by the cairo-runner.
/// e.g. cairo-runner --trace_file out/trace.bin
/// NOTE: The trace is not assumed to be padded to a power of 2 or relocated.
pub fn read_trace_bin(trace_path: PathBuf) -> Vec<Vec<Felt>> {
    let mut trace_elements: Vec<Vec<Felt>> = Default::default();

    let mut f = File::open(&trace_path)
        .expect("no file found");
    let metadata = fs::metadata(&trace_path)
        .expect("unable to read metadata");
    let length = metadata.len() as usize;

    println!("Trace:");
    
    let mut bytes_read = 0;
    let mut i = 0;

    let mut trace_items: Vec<TraceItem> = vec![];

    loop {
        let mut ap: [u8; 8] = Default::default();
        let mut fp: [u8; 8] = Default::default();
        let mut pc: [u8; 8] = Default::default();
        
        if bytes_read == length {
            break
        }
        
        bytes_read += f.read(&mut ap).unwrap();
        bytes_read += f.read(&mut fp).unwrap();
        bytes_read += f.read(&mut pc).unwrap();
        
        trace_items.push(TraceItem {
            ap: ap,
            fp: fp,
            pc: pc
        });

        println!(
            "{:#04x} ap={} fp={} pc={}", 
            i,
            hex::encode(ap), 
            hex::encode(fp),
            hex::encode(pc),
        );

        i += 1;

        trace_elements.push(
            [ap, fp, pc]
                .iter()
                .map(|x| Felt::try_from(u64::from_le_bytes(*x)).unwrap())
                .collect::<Vec<Felt>>()
        );
    }

    return trace_elements;
}


/// Parses a memory dump outputted by the cairo-runner.
/// e.g. cairo-runner --memory_file out/memory.bin
/// NOTE: The memory is not assumed to be relocated.
pub fn read_memory_bin(memory_path: PathBuf) -> Memory {
    let mut f = File::open(&memory_path)
        .expect("no file found");
    let metadata = fs::metadata(&memory_path)
        .expect("unable to read metadata");
    let length = metadata.len() as usize;

    println!("Memory:");
    
    let mut public_mem = Memory::new(vec![]).clone();

    let mut bytes_read = 0;
    let mut memory_items: Vec<MemoryItem> = vec![];

    // Load memory.bin.
    loop {
        if bytes_read == length {
            break
        }

        let mut address: [u8; 8] = Default::default();
        let mut value: [u8; 32]  = Default::default();

        bytes_read += f.read(&mut address).unwrap();
        address = address
            .iter()
            .map(|x| x.to_le_bytes()[0])
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();

        bytes_read += f.read(&mut value).unwrap();            
        value = value
            .iter()
            .map(|x| x.to_le_bytes()[0])
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();
        
        println!("{} {}", hex::encode(address), hex::encode(value));

        memory_items.push(MemoryItem {
            address: address,
            value: value
        });

        public_mem.write(
            Felt::try_from(u64::from_le_bytes(address)).unwrap(),
            Felt::try_from(&value).unwrap(),
        );
    }

    return public_mem;
}
