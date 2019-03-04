use std::io::{Seek, SeekFrom};
use storage::error::{Error, ErrorKind};
use storage::resource::{Resource, ResourcePtr};
use storage::view::ViewVec;
use storage::{Result, Size, Storage};

#[derive(Debug)]
pub struct Shard {
    pub start: usize,
    pub end: usize,
}

impl Size for Shard {
    #[inline(always)]
    fn size(&self) -> usize {
        if self.start > self.end {
            return 0;
        }
        return self.end - self.start;
    }
}

impl PartialEq for Shard {
    fn eq(&self, other: &Shard) -> bool {
        self.start == other.start && self.end == other.end
    }
}

pub trait Sharded: Storage {
    fn view(&self, start_idx: usize, size: usize) -> Result<ViewVec<<Self as Storage>::Ptr>>;

    fn seek(
        &self,
        resource: &mut <<Self as Storage>::Ptr as ResourcePtr>::Target,
        shard: &Shard,
    ) -> Result<()> {
        let start = SeekFrom::Start(shard.start as u64);
        let index = resource.handle().seek(start)? as usize;
        if index == shard.start {
            return Ok(());
        }

        err_new!(ErrorKind::InvalidOffset(shard.start))
    }
}

pub trait ShardReader: Storage {
    fn read_shard(
        &self,
        resource: &mut <<Self as Storage>::Ptr as ResourcePtr>::Target,
        shard: &Shard,
        into: &mut [u8],
    ) -> Result<usize>;
}

pub trait ShardWriter: Storage {
    fn write_shard(
        &self,
        resource: &mut <<Self as Storage>::Ptr as ResourcePtr>::Target,
        shard: &Shard,
        from: &[u8],
    ) -> Result<usize>;
}

impl<'s> From<&'s Shard> for Error {
    fn from(shard: &'s Shard) -> Self {
        Error::new(ErrorKind::InvalidOffsetAndSize(shard.start, shard.size()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_impl() {
        assert_eq!(Shard { start: 10, end: 10 }.size(), 0);
        assert_eq!(Shard { start: 15, end: 10 }.size(), 0);
        assert_eq!(Shard { start: 21, end: 31 }.size(), 10);
    }
}
