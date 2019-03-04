#[macro_use]
pub mod error;
#[macro_use]
pub mod generic;
#[macro_use]
pub mod file;

pub mod iter;
pub mod map;
pub mod resource;
pub mod shard;
pub mod view;
pub(crate) mod tests;

use storage::error::Error;
use storage::iter::StorageIterator;
use storage::resource::ResourcePtr;

pub type Result<T> = std::result::Result<T, Error>;
pub type StorageId = String;

pub trait Size {
    fn size(&self) -> usize;
}

pub trait Storage: Sized + Size {
    type Ptr: ResourcePtr;

    fn new(name: StorageId, items: Vec<(String, usize)>) -> Result<Self>;
    fn read(&self, offset: usize, into: &mut [u8]) -> Result<usize>;
    fn write(&self, offset: usize, from: &[u8]) -> Result<usize>;
    fn name(&self) -> &StorageId;

    fn iter(&self, chunk_size: usize) -> StorageIterator<Self> {
        StorageIterator::new(self, chunk_size)
    }
}
