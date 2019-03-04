extern crate bit_vec;
#[macro_use]
extern crate serde;
extern crate serde_bytes;

use bit_vec::BitVec;

#[derive(Serialize, Deserialize)]
#[serde(remote = "BitVec")]
pub struct BitVecSerde {
    #[serde(getter = "BitVec::to_bytes")]
    #[serde(with = "serde_bytes")]
    bytes: Vec<u8>,
}

impl From<BitVecSerde> for BitVec {
    fn from(definition: BitVecSerde) -> BitVec {
        BitVec::from_bytes(definition.bytes.as_ref())
    }
}
