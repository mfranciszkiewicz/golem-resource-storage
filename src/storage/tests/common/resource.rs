use storage::resource::Resource;
use storage::{Result, Size};

use super::handle::TestHandle;

#[derive(Debug)]
pub(crate) struct TestResource {
    test_handle: TestHandle,
    test_location: String,
    test_metadata: String,
}

impl Size for String {
    fn size(&self) -> usize {
        self.len()
    }
}

impl TestResource {
    fn new(location: &String, size: &usize) -> Result<Self> {
        let data: Vec<u8> = (0..*size as u64).map(|i| (i % 256) as u8).collect();

        let test_handle = TestHandle::new(data, *size);
        let test_location = location.clone();
        let test_metadata = "metadata".to_string();

        Ok(TestResource {
            test_handle,
            test_location,
            test_metadata,
        })
    }
}

impl Resource for TestResource {
    type Handle = TestHandle;
    type Metadata = String;

    fn open(location: &String) -> Result<Self> {
        let size = 65536 as usize;
        Self::new(location, &size)
    }

    fn create(location: &String, size: &usize) -> Result<Self> {
        Self::new(location, size)
    }

    fn exists(_location: &String) -> bool {
        return false;
    }

    fn metadata(_location: &String) -> Result<Self::Metadata> {
        Ok("Metadata".to_string())
    }

    fn handle(&mut self) -> &mut Self::Handle {
        &mut self.test_handle
    }

    fn location(&self) -> String {
        self.test_location.clone()
    }
}

impl Clone for TestResource {
    fn clone(&self) -> Self {
        TestResource::new(&self.test_location, &self.size()).unwrap()
    }
}

impl Size for TestResource {
    fn size(&self) -> usize {
        self.test_handle.size()
    }
}

impl_resource_serde!(TestResource);

#[cfg(test)]
pub(crate) mod tests {
    use std::io::Read;
    use storage::resource::Resource;
    use storage::tests::common::resource::TestResource;

    const SLICE_SIZE: usize = 256;
    const FILE_SIZE: usize = 65536;

    #[test]
    fn test_predefined_values() {
        let location = "location".to_string();
        let mut resource = TestResource::create(&location, &FILE_SIZE).unwrap();
        let handle = resource.handle();

        let expected: Vec<u8> = (0..256).map(|n| n as u8).collect();
        let mut read: [u8; SLICE_SIZE] = [0; SLICE_SIZE];

        (0..FILE_SIZE / SLICE_SIZE).for_each(|_| {
            handle.read_exact(&mut read).unwrap();
            assert_eq!(read[..], expected[..]);
        });
    }
}
