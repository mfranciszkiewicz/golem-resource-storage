use storage::Storage;
use streaming_iterator::StreamingIterator;

pub struct StorageIterator<'s, S>
where
    S: Storage + 's,
{
    storage: &'s S,
    size: usize,
    offset: usize,
    buf: Vec<u8>,
}

impl<'s, S> StorageIterator<'s, S>
where
    S: Storage + 's,
{
    pub fn new(storage: &'s S, size: usize) -> Self {
        StorageIterator {
            storage,
            size,
            offset: 0,
            buf: vec![0 as u8; size],
        }
    }
}

impl<'s, S> StreamingIterator for StorageIterator<'s, S>
where
    S: Storage + 's,
{
    type Item = Vec<u8>;

    fn advance(&mut self) {
        let read = self.storage.read(self.offset, &mut self.buf[..]);

        match read {
            Ok(n) => {
                self.offset += n;
                if n != self.size {
                    self.buf.truncate(n as usize);
                }
            }
            Err(_) => {
                self.offset = self.storage.size();
            }
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.offset >= self.size {
            None
        } else {
            Some(&self.buf)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.storage.size() + self.size - 1) / self.size;
        (0, Some(len))
    }
}
