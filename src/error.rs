use std::fmt;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Debug)]
pub struct Error<K>
where
    K: Debug + Send + Sync,
{
    pub kind: K,
}

impl<K> Error<K>
where
    K: Debug + Send + Sync
{
    pub fn new(kind: K) -> Self {
        Error { kind }
    }
}

impl<K> From<K> for Error<K>
where
    K: Debug + Send + Sync
{
    fn from(k: K) -> Self {
        Self::new(k)
    }
}

impl<K> Display for Error<K>
where
    K: Debug + Send + Sync
{
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{:?}", self.kind)?;
        Ok(())
    }
}

impl<K> std::error::Error for Error<K>
where
    K: Debug + Send + Sync
{}

#[macro_export]
macro_rules! err_new {
    ($kind:expr) => {
        Err($crate::error::Error::new($kind))
    };
}
