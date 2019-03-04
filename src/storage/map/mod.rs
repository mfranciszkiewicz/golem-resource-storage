pub mod chunk;
pub mod error;

use serde::{Deserialize, Serialize};
use merkle_tree::digest::sha512::Sha512;
use merkle_tree::proof::{Proof, Provable};
use merkle_tree::tree::MerkleTree;

use storage::{Storage, StorageId};
use self::chunk::ChunkMap;
use self::error::*;

#[derive(Serialize, Deserialize)]
pub struct StorageMap<S>
where
    S: Storage,
{
    tree: MerkleTree<Sha512>,
    chunks: ChunkMap,
    storage: S,
}

impl<S> StorageMap<S>
where
    S: Storage,
{
    pub fn new(name: StorageId, items: Vec<(String, usize)>) -> Result<Self, Error> {
        let storage = S::new(name, items)?;
        let chunks = ChunkMap::new(storage.size(), true);
        let tree = MerkleTree::<Sha512>::from(storage.iter(chunks.piece_size));

        Ok(StorageMap {
            tree,
            chunks,
            storage,
        })
    }

    #[inline]
    pub fn name(&self) -> &StorageId {
        self.storage.name()
    }

    pub fn read_chunk(&self, chunk: usize) -> Result<Vec<u8>, Error> {
        if !self.has_chunk(chunk) {
            return Err(Error::new(ErrorKind::ChunkDoesNotExist(chunk)));
        }

        let offset = chunk * self.chunks.chunk_size;
        self.read_storage(offset, self.chunks.chunk_size)
    }

    pub fn write_chunk(&mut self, chunk: usize, data: &Vec<u8>) -> Result<(), Error> {
        if self.has_chunk(chunk) {
            return Err(Error::new(ErrorKind::ChunkAlreadyExists(chunk)));
        }

        let offset = chunk * self.chunks.chunk_size;
        self.storage.write(offset, &data[..])?;
        self.chunks.bitmap.set(chunk, true);

        let piece_num = self.piece_from_chunk(chunk);
        if self.has_piece(piece_num) {
            self.update_tree(piece_num)?;
        }

        Ok(())
    }

    #[inline]
    pub fn has_chunk(&self, chunk_num: usize) -> bool {
        match self.chunks.bitmap.get(chunk_num) {
            Some(r) => r,
            None => false,
        }
    }

    pub fn has_piece(&self, piece_num: usize) -> bool {
        let first_chunk = (piece_num * self.chunks.piece_size) / self.chunks.chunk_size;
        (first_chunk..first_chunk + self.chunks.chunks_in_piece)
            .into_iter()
            .all(|i| self.has_chunk(i))
    }

    fn read_storage(&self, offset: usize, size: usize) -> Result<Vec<u8>, Error> {
        let mut buffer = vec![0 as u8; size];
        self.storage.read(offset, &mut buffer[..])?;
        Ok(buffer)
    }

    fn update_tree(&mut self, piece_num: usize) -> Result<(), Error> {
        let offset = piece_num * self.chunks.piece_size;
        let buffer = self.read_storage(offset, self.chunks.piece_size)?;
        self.tree.set(piece_num, &buffer)?;
        Ok(())
    }

    #[inline]
    fn piece_from_chunk(&self, chunk_num: usize) -> usize {
        (chunk_num * self.chunks.chunk_size) / self.chunks.piece_size
    }
}

impl<S> Provable<Error> for StorageMap<S>
where
    S: Storage,
{
    fn prove(&self, leaf_index: usize) -> Result<Proof, Error> {
        let proof = self.tree.prove(leaf_index)?;
        Ok(proof)
    }

    fn verify(&self, proof: &Proof) -> Result<(), Error> {
        self.tree.verify(proof)?;
        Ok(())
    }
}
