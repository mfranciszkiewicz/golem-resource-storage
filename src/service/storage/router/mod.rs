use std::collections::HashMap;

use actix::fut::{err, ok, wrap_future};
use actix::*;

use futures::future::Future;
use service::error::{Error, ErrorKind};
use service::storage::map::StorageMapActor;
use service::storage::message::*;

pub struct StorageRouter {
    actors: HashMap<String, Addr<StorageMapActor>>,
}

impl StorageRouter {
    fn spawn(&mut self, name: String) -> Addr<StorageMapActor> {
        let address: Addr<StorageMapActor> = Arbiter::start(move |_| StorageMapActor::new());
        let _ = self.actors.insert(name, address.clone());
        address
    }
}

impl Actor for StorageRouter {
    type Context = Context<Self>;
}

macro_rules! err {
    ($kind:expr) => {
        Box::new(err(Error::new($kind)))
    };
}

macro_rules! wrap_future {
    ($Self:tt, $forward:expr) => {
        wrap_future::<_, $Self>($forward)
            .map(|result, _a, _c| result.unwrap())
            .then(|result, _a, _c| match result {
                Ok(v) => ok(v),
                Err(e) => err(e),
            })
    }
}

macro_rules! impl_forward_new {
    ($Message:tt) => {
        impl Handler<$Message> for StorageRouter {
            type Result = ResponseActFuture<Self, <$Message as ValueHint>::Value, Error>;

            fn handle(&mut self, msg: $Message, _ctx: &mut Self::Context) -> Self::Result {
                if let Some(_) = self.actors.get(&msg.id) {
                    return err!(ErrorKind::StorageAlreadyExists);
                }

                let address = self.spawn(msg.id.clone());
                let future = wrap_future!(Self, address.send(msg).map_err(Error::from));
                Box::new(future)
            }
        }
    }
}

macro_rules! impl_forward {
    ($Message:tt) => {
        impl Handler<$Message> for StorageRouter {
            type Result = ResponseActFuture<Self, <$Message as ValueHint>::Value, Error>;

            fn handle(&mut self, msg: $Message, _ctx: &mut Self::Context) -> Self::Result {
                let address = match self.actors.get(&msg.id) {
                    Some(address) => address,
                    None => return err!(ErrorKind::StorageDoesNotExist),
                };

                let send = address.send(msg).map_err(Error::from);
                Box::new(wrap_future!(Self, send))
            }
        }
    }
}

impl_forward_new!(Create);
impl_forward_new!(Load);

impl_forward!(Save);
impl_forward!(ReadChunk);
impl_forward!(WriteChunk);
impl_forward!(HasChunk);
impl_forward!(HasPiece);
impl_forward!(Prove);
impl_forward!(VerifyProof);
