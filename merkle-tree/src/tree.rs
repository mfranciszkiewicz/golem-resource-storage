use std::marker::PhantomData;

use bit_vec::BitVec;
use bit_vec_serde::BitVecSerde;
use serde::{Deserialize, Serialize};
use streaming_iterator::StreamingIterator;

use digest::Digest;
use level::IndexedLevel;
use proof;
use proof::error::{Error, ErrorKind};
use proof::{Proof, Provable};

use {Array, Result};

#[derive(Serialize, Deserialize)]
pub struct MerkleTree<D>
where
    D: Digest,
{
    /// tree nodes that have been set
    #[serde(with = "BitVecSerde")]
    bitmap: BitVec,
    /// tree node hashes
    #[serde(with = "serde_bytes")]
    hashes: Vec<u8>,
    /// tree height (levels)
    height: usize,
    /// tree leaf count
    leaf_count: usize,
    /// type holder
    phantom: PhantomData<D>,
}

impl<D> MerkleTree<D>
where
    D: Digest,
{
    fn new(leaf_count: usize) -> Self {
        let (size, height) = tree_size(leaf_count);
        let hashes = vec![0 as u8; size * D::output_size()];
        let bitmap = BitVec::from_elem(size, false);

        MerkleTree {
            bitmap,
            hashes,
            height,
            leaf_count,
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn has(&self, index: usize) -> bool {
        match self.bitmap.get(index) {
            Some(b) => b,
            None => false,
        }
    }

    pub fn get(&self, leaf_index: usize) -> Result<Vec<u8>> {
        if leaf_index >= self.leaf_count {
            return err!("Leaf index {:?} out of range", leaf_index);
        }

        let hash = self.get_hash(leaf_index);
        Ok(hash.to_vec())
    }

    pub fn set(&mut self, leaf_index: usize, hash: &Array) -> Result<()> {
        if leaf_index >= self.leaf_count {
            return err!("Leaf index {:?} out of range", leaf_index);
        }

        self.set_hash(leaf_index, &hash);
        self.build_down(leaf_index);

        Ok(())
    }

    #[inline(always)]
    pub fn built(&self) -> bool {
        self.bitmap.all()
    }

    #[inline]
    fn get_hash(&self, index: usize) -> &[u8] {
        let byte_index = index * D::output_size();
        &self.hashes[byte_index..byte_index + D::output_size()]
    }

    fn set_hash(&mut self, index: usize, hash: &Array) {
        // update hash for node at index
        let byte_index = index * D::output_size();
        let slice = &mut self.hashes[byte_index..byte_index + D::output_size()];
        slice.clone_from_slice(&hash.as_ref());

        // mark node at index as set
        self.bitmap.set(index, true);
    }

    #[inline]
    fn build(&mut self) {
        for i in (0..self.leaf_count).step_by(2) {
            self.build_down(i);
        }
    }

    fn build_down(&mut self, leaf_index: usize) {
        let mut digest = D::new();
        let mut ilevel = IndexedLevel::new(leaf_index, 0, self.leaf_count).unwrap();

        for _ in 0..self.height - 1 {
            for sibling in ilevel.siblings().iter() {
                if let Some(index) = sibling {
                    if !self.has(*index) {
                        return;
                    }
                    digest.input(&self.get_hash(*index));
                }
            }

            self.set_hash(ilevel.parent(), &digest.result());
            ilevel = ilevel.down().unwrap();
        }
    }
}

impl<D, I> From<I> for MerkleTree<D>
where
    D: Digest,
    I: StreamingIterator<Item = Array>,
{
    fn from(input: I) -> Self {
        let mut hashes = build_leaves::<I, D>(input);
        let leaf_count = hashes.len() / D::output_size();

        let (size, height) = tree_size(leaf_count);
        hashes.resize(size * D::output_size(), 0 as u8);

        let mut bitmap = BitVec::from_elem(leaf_count, true);
        bitmap.grow(size - leaf_count, false);

        let mut tree = MerkleTree {
            bitmap,
            hashes,
            height,
            leaf_count,
            phantom: PhantomData,
        };

        tree.build();
        tree
    }
}

impl<D> Provable<Error> for MerkleTree<D>
where
    D: Digest,
{
    fn prove(&self, leaf_index: usize) -> proof::Result<Proof> {
        let mut path = Vec::with_capacity(self.height);
        let mut ilevel = IndexedLevel::new(leaf_index, 0, self.leaf_count).unwrap();

        for _ in 0..self.height {
            let entry = match ilevel.sibling() {
                Some(index) => {
                    if self.has(index) {
                        Some(self.get_hash(index).to_vec())
                    } else {
                        break;
                    }
                }
                None => None,
            };

            path.push(entry);
            ilevel = ilevel.down().unwrap();
        }

        if path.len() < 2 {
            return proof_err!(ErrorKind::InvalidLength, path.len());
        }

        Ok(Proof {
            leaf_index,
            leaf_hash: self.get_hash(leaf_index).to_vec(),
            path,
            partial: !self.built(),
        })
    }

    fn verify(&self, proof: &Proof) -> proof::Result<()> {
        if proof.leaf_index >= self.leaf_count {
            return proof_err!(ErrorKind::IndexOutOfRange, proof.leaf_index);
        }
        if proof.path.len() < 2 {
            return proof_err!(ErrorKind::InvalidLength, proof.path.len());
        }
        if self.get_hash(proof.leaf_index) != &proof.leaf_hash[..] {
            return proof_err!(ErrorKind::InvalidHash, proof.leaf_index);
        }

        <Self as Provable<Error>>::prove(&self, proof.leaf_index)?.validate(&proof)
    }
}

fn build_leaves<I, D>(mut iter: I) -> Vec<u8>
where
    D: Digest,
    I: StreamingIterator<Item = Array>,
{
    let mut digest = D::new();
    let mut leaves = match iter.size_hint().1 {
        Some(size) => Vec::with_capacity(size * D::output_size()),
        None => Vec::new(),
    };

    while let Some(data) = iter.next() {
        digest.input(data);
        leaves.extend_from_slice(&digest.result()[..]);
    }

    leaves
}

fn tree_size(mut leaf_count: usize) -> (usize, usize) {
    let mut height = 0;
    let mut sum = 0;

    loop {
        height += 1;
        sum += leaf_count;
        if leaf_count <= 1 {
            break;
        }

        leaf_count = (leaf_count + 1) >> 1;
    }

    if height == 1 {
        sum += 1;
        height += 1;
    }

    (sum, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    use digest::sha512::Sha512;
    use level::Level;
    use proof::Provable;
    use rand::Rng;

    type D = Sha512;

    fn random_leaves(leaf_count: usize) -> Vec<Array> {
        (0..leaf_count)
            .map(|_| {
                let mut buf = [0 as u8; 1024];
                rand::thread_rng().fill(&mut buf[..]);
                buf.to_vec()
            })
            .collect()
    }

    fn lower_level_digests(digests: &Vec<Array>) -> Vec<Array> {
        let mut digest = D::new();
        let mut result: Vec<Array> = Vec::new();

        for i in (0..digests.len()).step_by(2) {
            digest.input(&digests[i][..]);
            if i + 1 < digests.len() {
                digest.input(&digests[i + 1][..]);
            }
            result.push(digest.result());
        }

        result
    }

    fn digests_to_bytes(source: &Vec<Array>) -> Array {
        let mut bytes = Array::new();
        source.iter().for_each(|l| {
            bytes.extend_from_slice(&l[..]);
        });

        bytes
    }

    #[test]
    fn test_new() {
        let mut tree;

        tree = MerkleTree::<D>::new(1);
        assert_eq!(tree.leaf_count, 1);
        assert_eq!(tree.height, 2);

        tree = MerkleTree::<D>::new(2);
        assert_eq!(tree.leaf_count, 2);
        assert_eq!(tree.height, 2);

        tree = MerkleTree::<D>::new(3);
        assert_eq!(tree.leaf_count, 3);
        assert_eq!(tree.height, 3);

        tree = MerkleTree::<D>::new(3);
        assert_eq!(tree.leaf_count, 3);
        assert_eq!(tree.height, 3);
    }

    #[test]
    fn test_build() {
        let leaf_count = 10;

        let leaves: Vec<Array> = random_leaves(leaf_count);
        let mut digest = D::new();
        let mut digests: Vec<Array> = leaves
            .iter()
            .map(|e| {
                digest.input(&e[..]);
                digest.result().to_vec()
            })
            .collect();

        let mut level = Level::new(0, leaves.len());
        let tree = MerkleTree::<D>::from(leaves.iter());
        assert_eq!(tree.built(), true);
        assert_eq!(tree.leaf_count, leaf_count);
        assert_eq!(tree.height, 5);

        for _ in 0..tree.height {
            let start = level.start * D::output_size();
            let end = level.end * D::output_size();
            let bytes = digests_to_bytes(&digests);

            assert_eq!(bytes.len(), end - start);
            assert_eq!(bytes[..], tree.hashes[start..end]);

            match level.down() {
                Some(lower) => {
                    level = lower;
                    digests = lower_level_digests(&digests);
                }
                None => break,
            };
        }
    }

    #[test]
    fn test_create_proof() {
        let mut tree;
        let mut proof;

        for leaf_count in [1 as usize, 10, 13].iter() {
            tree = MerkleTree::<D>::from(random_leaves(*leaf_count).iter());
            proof = tree.prove(leaf_count - 1).unwrap();

            assert_eq!(proof.leaf_index, leaf_count - 1);
            assert_eq!(proof.path.len(), tree.height);
            assert_eq!(proof.partial, false);
        }
    }

    #[test]
    fn test_verify_proof() {
        for leaf_count in [1 as usize, 10, 13].iter() {
            let tree = MerkleTree::<D>::from(random_leaves(*leaf_count).iter());
            for leaf in 0..*leaf_count {
                let proof = tree.prove(leaf).unwrap();
                tree.verify(&proof).unwrap();
            }
        }
    }

    #[test]
    fn test_verify_partial_proof() {
        let leaf_count = 10;
        let tree = MerkleTree::<D>::from(random_leaves(leaf_count).iter());

        for leaf in 0..leaf_count {
            let mut proof = tree.prove(leaf).unwrap();
            {
                let len = proof.path.len();
                proof.partial = true;
                proof.path.remove(len - 1);
            }

            match tree.verify(&proof) {
                Ok(()) => panic!("Partial proof verification should return an error"),
                Err(err) => {
                    if err.kind != ErrorKind::PartialProof {
                        panic!("Verification failed with {:?}", err.kind);
                    }
                }
            }
        }
    }

    #[test]
    fn test_verify_errors() {
        let leaf_count = 10;
        let tree = MerkleTree::<D>::from(random_leaves(leaf_count).iter());
        let verify = |proof: &Proof, kind: ErrorKind| match tree.verify(&proof) {
            Ok(()) => panic!("Proof verification should return an error"),
            Err(err) => {
                if err.kind != kind {
                    panic!("Verification failed with {:?}", err.kind);
                }
            }
        };

        let mut proof = Proof {
            leaf_index: 10,
            leaf_hash: Array::new(),
            path: vec![Some(Array::new())],
            partial: true,
        };
        verify(&proof, ErrorKind::IndexOutOfRange);

        proof.leaf_index = 1;
        verify(&proof, ErrorKind::InvalidLength);

        proof.path = vec![Some(Array::new()), Some(Array::new())];
        verify(&proof, ErrorKind::InvalidHash);
    }
}
