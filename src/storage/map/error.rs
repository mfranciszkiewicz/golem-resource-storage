use error;
use storage::error::ErrorKind as StorageErrorKind;

#[derive(Debug)]
pub enum ErrorKind {
    ChunkAlreadyExists(usize),
    ChunkDoesNotExist(usize),
    ChunkOutOfRange(usize),
    StorageError(StorageErrorKind),
    MerkleTreeError(merkle_tree::error::Error),
    MerkleTreeProofError(merkle_tree::proof::error::Error),
    IoError(String),
}

pub type Error = error::Error<ErrorKind>;

impl From<error::Error<StorageErrorKind>> for Error {
    fn from(error: error::Error<StorageErrorKind>) -> Self {
        let kind = if let StorageErrorKind::IoError(s) = error.kind {
            ErrorKind::IoError(s)
        } else {
            ErrorKind::StorageError(error.kind)
        };

        Error::new(kind)
    }
}

impl From<merkle_tree::error::Error> for Error {
    fn from(error: merkle_tree::error::Error) -> Self {
        Error::new(ErrorKind::MerkleTreeError(error))
    }
}

impl From<merkle_tree::proof::error::Error> for Error {
    fn from(error: merkle_tree::proof::error::Error) -> Self {
        Error::new(ErrorKind::MerkleTreeProofError(error))
    }
}
