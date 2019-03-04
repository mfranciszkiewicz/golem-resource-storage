use actix::*;
use merkle_tree::proof::Proof;
use service::error::Error;

pub type Array = Vec<u8>;

pub trait ValueHint {
    type Value;
}

macro_rules! impl_message {
    ($tt:tt, $v:tt) => {
        impl ValueHint for $tt {
            type Value = $v;
        }

        impl Message for $tt {
            type Result = Result<$v, Error>;
        }
    };
}

pub struct Create {
    pub id: String,
    pub resources: Vec<(String, usize)>,
}

pub struct Load {
    pub id: String,
    pub location: String,
}

pub struct Save {
    pub id: String,
    pub location: String,
}

pub struct ReadChunk {
    pub id: String,
    pub chunk: usize,
}

pub struct WriteChunk {
    pub id: String,
    pub chunk: usize,
    pub data: Vec<u8>,
}

pub struct HasChunk {
    pub id: String,
    pub chunk: usize,
}

pub struct HasPiece {
    pub id: String,
    pub piece: usize,
}

pub struct Prove {
    pub id: String,
    pub leaf_index: usize,
}

pub struct VerifyProof {
    pub id: String,
    pub proof: Proof,
}

impl_message!(Create, String);
impl_message!(Load, String);
impl_message!(Save, ());
impl_message!(ReadChunk, Array);
impl_message!(WriteChunk, ());
impl_message!(HasChunk, bool);
impl_message!(HasPiece, bool);
impl_message!(Prove, Proof);
impl_message!(VerifyProof, ());
