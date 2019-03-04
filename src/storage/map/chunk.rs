use std::cmp::{max, min};

use bit_vec::BitVec;
use bit_vec_serde::BitVecSerde;
use serde::{Deserialize, Serialize};

#[inline(always)]
fn div_upper(value: usize, by: usize) -> usize {
    (value + by - 1) / by
}

#[derive(Serialize, Deserialize)]
pub(super) struct ChunkMap {
    #[serde(with = "BitVecSerde")]
    pub bitmap: BitVec,
    pub chunk_size: usize,
    pub chunk_count: usize,
    pub piece_size: usize,
    pub piece_count: usize,
    pub chunks_in_piece: usize,
}

impl ChunkMap {
    pub const MIN_PIECE_SIZE: usize = 16384;
    pub const MAX_PIECE_SIZE: usize = 1 << 10;

    pub fn new(total_size: usize, all_set: bool) -> Self {
        let piece_size = Self::piece_size(total_size);
        let chunk_size = Self::chunk_size(piece_size);
        let piece_count = div_upper(total_size, piece_size);
        let chunks_in_piece = div_upper(piece_size, chunk_size);
        let chunk_count = piece_count * chunks_in_piece;
        let bitmap = BitVec::from_elem(chunk_count, all_set);

        ChunkMap {
            bitmap,
            chunk_size,
            chunk_count,
            piece_size,
            piece_count,
            chunks_in_piece,
        }
    }

    fn piece_size(total_size: usize) -> usize {
        let value = if total_size > 0 {
            let bits: usize = std::mem::size_of::<usize>() * 8;
            let zeros: usize = total_size.leading_zeros() as usize;
            1 << (bits - zeros - 1)
        } else {
            0
        };
        max(min(value, Self::MAX_PIECE_SIZE), Self::MIN_PIECE_SIZE)
    }

    #[inline(always)]
    fn chunk_size(piece_size: usize) -> usize {
        piece_size >> 2
    }
}