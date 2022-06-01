use serde::{Deserialize, Serialize};

pub mod prove;
pub mod verify;

#[derive(Serialize, Deserialize)]
struct ProofData {
    input_bytes: Vec<u8>,
    proof_bytes: Vec<u8>,
}
