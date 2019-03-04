use std::cell::BorrowMutError;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use storage::error::Error;
use storage::error::ErrorKind;
use storage::resource::*;
use storage::Size;

#[macro_export]
macro_rules! impl_resource_serde {
    ($res_type:ident) => {
        impl serde::Serialize for $res_type {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.location().as_str())
            }
        }

        impl<'de> serde::Deserialize<'de> for $res_type {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let location = String::deserialize(deserializer)?;
                match <Self as crate::storage::resource::Resource>::open(&location) {
                    Ok(res) => Ok(res),
                    Err(err) => Err(serde::de::Error::custom(err)),
                }
            }
        }
    };
}

pub type GenericResourcePtr<R> = Rc<RefCell<R>>;

impl<R> ResourcePtr for Rc<RefCell<R>>
where
    R: Resource,
{
    type Target = R;

    fn new(r: Self::Target) -> Self {
        Rc::new(RefCell::new(r))
    }
}

impl<R> Size for Rc<RefCell<R>>
where
    R: Resource,
{
    fn size(&self) -> usize {
        (*self).borrow().deref().size()
    }
}

impl<R> Size for RefCell<R>
where
    R: Resource,
{
    fn size(&self) -> usize {
        (*self).borrow().size()
    }
}

impl From<BorrowMutError> for Error {
    fn from(_: BorrowMutError) -> Self {
        Error::new(ErrorKind::MemoryError(
            "Resource pointer error: already borrowed as mutable".to_string(),
        ))
    }
}
