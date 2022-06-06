use air::{ProcessorAir, PublicInputs};
use serde::{Deserialize, Serialize};
use winter_utils::{Deserializable, SliceReader};
use winterfell::StarkProof;

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn verify(buffer: &Uint8Array) {
    // Load proof and public inputs
    let b = buffer.to_vec();
    let data: ProofData = bincode::deserialize(&b).unwrap();
    let pub_inputs = PublicInputs::read_from(&mut SliceReader::new(&data.input_bytes[..])).unwrap();
    let proof = StarkProof::from_bytes(&data.proof_bytes).unwrap();

    // Verify execution
    match winterfell::verify::<ProcessorAir>(proof, pub_inputs) {
        Ok(_) => log("Execution verified"),
        Err(err) => log(format!("Failed to verify execution: {}", err).as_str()),
    }
}

#[derive(Serialize, Deserialize)]
struct ProofData {
    input_bytes: Vec<u8>,
    proof_bytes: Vec<u8>,
}
