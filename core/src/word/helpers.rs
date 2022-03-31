// Modified from https://github.com/o1-labs/proof-systems

use super::{super::StarkField, Felt};
use winter_utils::AsBytes;

pub trait FieldHelpers {
    /// Return field element as byte, if it fits. Otherwise returns least significant byte
    fn lsb(self) -> u8;

    /// Return pos-th 16-bit chunk as another field element
    fn chunk_u16(self, pos: usize) -> Felt;

    /// Return first 64 bits of the field element
    fn to_u64(self) -> u64;

    /// Return a field element in hexadecimal in little endian
    fn to_hex_le(self) -> String;

    /// Return a vector of field elements from a vector of i128
    fn vec_to_field(vec: &[i128]) -> Vec<Felt>;

    /// Return a vector of bits
    fn to_bits(self) -> Vec<bool>;
}

impl FieldHelpers for Felt {
    fn lsb(self) -> u8 {
        self.as_bytes()[0]
    }

    fn chunk_u16(self, pos: usize) -> Felt {
        let bytes = self.as_bytes();
        let chunk = u16::from(bytes[2 * pos]) + u16::from(bytes[2 * pos + 1]) * 2u16.pow(8);
        Felt::from(chunk)
    }

    fn to_u64(self) -> u64 {
        let bytes = self.as_bytes();
        let mut acc: u64 = 0;
        for i in 0..8 {
            acc += 2u64.pow(i * 8) * (bytes[i as usize] as u64);
        }
        acc
    }

    fn to_hex_le(self) -> String {
        let mut bytes = self.as_int().to_le_bytes();
        bytes.reverse();
        hex::encode(bytes)
    }

    fn vec_to_field(vec: &[i128]) -> Vec<Felt> {
        vec.iter()
            .map(|i| {
                if *i < 0 {
                    -Felt::from((-(*i)) as u64)
                } else {
                    Felt::from((*i) as u64)
                }
            })
            .collect()
    }

    fn to_bits(self) -> Vec<bool> {
        self.as_bytes().iter().fold(vec![], |mut bits, byte| {
            let mut byte = *byte;
            for _ in 0..8 {
                bits.push(byte & 0x01 == 0x01);
                byte >>= 1;
            }
            bits
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_to_bits() {
        let fe = Felt::from(256u32);
        let bits = fe.to_bits();
        println!("{:?}", &bits[0..16]);
    }

    #[test]
    fn test_field_to_chunks() {
        let fe = Felt::from(0x480680017fff8000u64);
        let chunk = fe.chunk_u16(1);
        println!("chunk {:?}", chunk);
        println!("chunk2 {:?}", Felt::from(0x7fffu64));
        assert_eq!(chunk, Felt::from(0x7fffu64));
    }

    //#[test]
    //fn test_hex_and_u64() {
    //    let fe = Felt::from(0x480680017fff8000u64);
    //    let change = Felt::from(&fe.to_hex()).unwrap();
    //    assert_eq!(fe, change);
    //    let word = change.to_u64();
    //    assert_eq!(word, 0x480680017fff8000u64);
    //}
}
