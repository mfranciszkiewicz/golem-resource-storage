macro_rules! err {
    ($($arg:tt)*) => (Err($crate::error::Error::new(format!($($arg)*))))
}

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new<M>(message: M) -> Self
    where
        Error: From<M>,
    {
        Self::from(message)
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Error { message }
    }
}

impl From<&String> for Error {
    fn from(message: &String) -> Self {
        Error {
            message: message.clone(),
        }
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Error {
            message: String::from(message),
        }
    }
}
