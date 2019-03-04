use std::io;

#[derive(Debug)]
pub enum ErrorKind {
    SizeMismatch(usize, usize),
    InvalidOffset(usize),
    InvalidOffsetAndSize(usize, usize),
    InvalidView,
    ViewBuildError(usize, usize, usize),
    LocationError(String),
    MemoryError(String),
    IoError(String),
    Custom(String),
}

impl Clone for ErrorKind {
    fn clone(&self) -> Self {
        match self {
            ErrorKind::SizeMismatch(l, r) => ErrorKind::SizeMismatch(*l, *r),
            ErrorKind::InvalidOffset(o) => ErrorKind::InvalidOffset(*o),
            ErrorKind::InvalidOffsetAndSize(o, s) => ErrorKind::InvalidOffsetAndSize(*o, *s),
            ErrorKind::InvalidView => ErrorKind::InvalidView,
            ErrorKind::ViewBuildError(s, e, o) => ErrorKind::ViewBuildError(*s, *e, *o),
            ErrorKind::LocationError(s) => ErrorKind::LocationError(s.clone()),
            ErrorKind::MemoryError(s) => ErrorKind::MemoryError(s.clone()),
            ErrorKind::IoError(error) => ErrorKind::IoError(format!("{:?}", error)),
            ErrorKind::Custom(s) => ErrorKind::Custom(s.clone()),
        }
    }
}

pub type Error = crate::error::Error<ErrorKind>;

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::new(ErrorKind::IoError(format!("{:?}", error)))
    }
}

impl<'s> From<&'s str> for Error {
    fn from(string: &'s str) -> Self {
        Error::new(ErrorKind::Custom(string.to_string()))
    }
}

impl<'s> From<&'s String> for Error {
    fn from(string: &'s String) -> Self {
        Error::new(ErrorKind::Custom(string.clone()))
    }
}
