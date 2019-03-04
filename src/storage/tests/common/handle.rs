use std::io;
use std::io::{Read, Seek, Write};

use storage::Size;

#[derive(Debug)]
pub struct TestHandle {
    inner: io::Cursor<Vec<u8>>,
    inner_size: usize,
}

impl TestHandle {
    pub fn new(data: Vec<u8>, size: usize) -> Self {
        TestHandle {
            inner: io::Cursor::new(data),
            inner_size: size,
        }
    }
}

impl Read for TestHandle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Write for TestHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl Seek for TestHandle {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl Size for TestHandle {
    fn size(&self) -> usize {
        self.inner_size
    }
}
