#[macro_use]
pub mod error;

use self::error::{Error, ErrorKind};
use serde::{Deserialize, Serialize};
use Array;

pub type Result<T> = std::result::Result<T, Error>;

pub trait Provable<E> {
    fn prove(&self, leaf_index: usize) -> std::result::Result<Proof, E>;
    fn verify(&self, proof: &Proof) -> std::result::Result<(), E>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Proof {
    pub leaf_index: usize,
    pub leaf_hash: Array,
    pub path: Vec<Option<Array>>,
    pub partial: bool,
}

impl Proof {
    pub fn validate(&self, other: &Proof) -> Result<()> {
        if self.leaf_index != other.leaf_index {
            return proof_err!(ErrorKind::InvalidIndex, other.leaf_index);
        }
        if !self.partial && !other.partial {
            if self.path.len() != other.path.len() {
                return proof_err!(ErrorKind::InvalidLength, other.path.len());
            }
        }

        let end = std::cmp::min(self.path.len(), other.path.len());

        if self.path[..end] != other.path[..end] {
            return proof_err!(ErrorKind::InvalidHash, "hash mismatch in proof");
        }
        if self.partial != other.partial {
            return proof_err!(ErrorKind::PartialProof, "validated partially");
        }
        Ok(())
    }
}
