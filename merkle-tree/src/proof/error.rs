macro_rules! proof_err {
    ($kind:expr, $message:expr) => {
        Err($crate::proof::error::Error::new($kind, $message))
    };
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    IndexOutOfRange,
    InvalidLength,
    InvalidIndex,
    InvalidHash,
    PartialProof,
}

#[derive(Clone, Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

impl Error {
    pub fn new<S>(kind: ErrorKind, message: S) -> Self
    where
        S: std::fmt::Debug,
    {
        Error {
            kind,
            message: format!("{:?}", message),
        }
    }
}
