use std::fmt;
use std::io::{Read, Seek, Write};

use storage::{Result, Size};

pub trait Resource: Clone + fmt::Debug + Size + Sized {
    type Handle: Read + Seek + Write;
    type Metadata: Size;

    fn open(location: &String) -> Result<Self>;
    fn create(location: &String, size: &usize) -> Result<Self>;
    fn exists(location: &String) -> bool;
    fn metadata(location: &String) -> Result<Self::Metadata>;

    fn handle(&mut self) -> &mut Self::Handle;
    fn location(&self) -> String;
}

pub trait ResourcePtr: Clone + fmt::Debug + Size {
    type Target: Resource;

    fn new(r: Self::Target) -> Self;
}
