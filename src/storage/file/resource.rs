use std::fs::{create_dir_all, File, Metadata, OpenOptions};
use std::path::Path;

use fs2::FileExt;

use storage::error::{Error, ErrorKind};
use storage::resource::Resource;
use storage::{Result, Size};

impl Size for Metadata {
    fn size(&self) -> usize {
        self.len() as usize
    }
}

impl<'p> From<&'p Path> for Error {
    fn from(path: &'p Path) -> Self {
        Error::new(ErrorKind::LocationError(path.display().to_string()))
    }
}

#[derive(Debug)]
pub struct FileResource {
    file_handle: File,
    file_location: String,
    file_size: usize,
}

impl FileResource {
    pub fn new(handle: File, location: &String, size: usize) -> Self {
        FileResource {
            file_handle: handle,
            file_location: location.clone(),
            file_size: size,
        }
    }

    fn try_from(handle: File, location: &String) -> Result<Self> {
        let size = handle.allocated_size()? as usize;

        Ok(FileResource::new(handle, location, size))
    }

    fn open(location: &String, create: bool) -> Result<<Self as Resource>::Handle> {
        let path = Path::new(location);
        let file = OpenOptions::new()
            .create(create)
            .read(true)
            .write(true)
            .append(false)
            .truncate(false)
            .open(path)?;

        Ok(file)
    }
}

impl Resource for FileResource {
    type Handle = File;
    type Metadata = Metadata;

    fn open(location: &String) -> Result<Self> {
        let handle = FileResource::open(location, false)?;
        FileResource::try_from(handle, location)
    }

    fn create(location: &String, size: &usize) -> Result<Self> {
        if let Some(parent) = Path::new(location).parent() {
            create_dir_all(parent)?;
        }

        let file = FileResource::open(location, true)?;
        file.allocate(*size as u64)?;

        FileResource::try_from(file, location)
    }

    #[inline(always)]
    fn exists(location: &String) -> bool {
        Path::new(location).exists()
    }

    #[inline(always)]
    fn metadata(location: &String) -> Result<Self::Metadata> {
        let result = Path::new(location).metadata()?;
        Ok(result)
    }

    #[inline(always)]
    fn handle(&mut self) -> &mut Self::Handle {
        &mut self.file_handle
    }

    #[inline(always)]
    fn location(&self) -> String {
        self.file_location.clone()
    }
}

impl Clone for FileResource {
    fn clone(&self) -> Self {
        let handle = self.file_handle.try_clone().unwrap();
        FileResource::new(handle, &self.file_location, self.file_size)
    }
}

impl Size for FileResource {
    #[inline(always)]
    fn size(&self) -> usize {
        self.file_size
    }
}

impl_resource_serde!(FileResource);
