use std::io;

use actix::MailboxError;
use bincode;
use error;
use storage::error::ErrorKind as StorageErrorKind;
use storage::map::error::ErrorKind as StorageMapErrorKind;

#[derive(Debug)]
pub enum ErrorKind {
    IoError(String),
    BincodeError(bincode::Error),
    StorageError(StorageErrorKind),
    StorageMapError(StorageMapErrorKind),
    StorageAlreadyExists,
    StorageDoesNotExist,
    MailboxError(MailboxError),
}

pub type Error = error::Error<ErrorKind>;

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::new(ErrorKind::IoError(format!("{:?}", error)))
    }
}

impl From<bincode::Error> for Error {
    fn from(error: bincode::Error) -> Self {
        Self::new(ErrorKind::BincodeError(error))
    }
}

impl From<StorageErrorKind> for Error {
    fn from(kind: StorageErrorKind) -> Self {
        Self::new(ErrorKind::StorageError(kind))
    }
}

impl From<error::Error<StorageErrorKind>> for Error {
    fn from(error: error::Error<StorageErrorKind>) -> Self {
        Self::new(ErrorKind::StorageError(error.kind))
    }
}

impl From<error::Error<StorageMapErrorKind>> for Error {
    fn from(error: error::Error<StorageMapErrorKind>) -> Self {
        let kind = if let StorageMapErrorKind::IoError(s) = error.kind {
            ErrorKind::IoError(s)
        } else if let StorageMapErrorKind::StorageError(kind) = error.kind {
            ErrorKind::StorageError(kind)
        } else {
            ErrorKind::StorageMapError(error.kind)
        };

        Error::new(kind)
    }
}

impl From<MailboxError> for Error {
    fn from(error: MailboxError) -> Self {
        Error::new(ErrorKind::MailboxError(error))
    }
}
