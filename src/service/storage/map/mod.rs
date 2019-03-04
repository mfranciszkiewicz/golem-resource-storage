mod serialize;
mod version;

use std::path::Path;

use actix::*;
use merkle_tree::proof::Provable;

use self::serialize::{deserialize_from, serialize_into};
use service::error::{Error, ErrorKind};
use service::storage::map::version::{StorageMapVersion, VersionedStorageMap};
use service::storage::message;
use service::Result;

pub struct StorageMapActor {
    holder: Option<VersionedStorageMap>,
}

impl StorageMapActor {
    pub fn new() -> Self {
        StorageMapActor { holder: None }
    }

    fn create(name: String, resources: Vec<(String, usize)>) -> Result<VersionedStorageMap> {
        let storage_map = StorageMapVersion::new(name, resources)?;
        let holder = VersionedStorageMap::wrap(storage_map);
        Ok(holder)
    }

    fn load(location: &String) -> Result<VersionedStorageMap> {
        let path = Path::new(location);
        let holder = deserialize_from::<VersionedStorageMap>(path)?;
        Ok(holder)
    }

    fn try_unwrap(&self) -> Result<&StorageMapVersion> {
        match &self.holder {
            Some(h) => h.try_unwrap(),
            None => Err(Error::new(ErrorKind::StorageDoesNotExist)),
        }
    }
}

impl From<StorageMapVersion> for StorageMapActor {
    fn from(map: StorageMapVersion) -> Self {
        Self {
            holder: Some(VersionedStorageMap::V1(map)),
        }
    }
}

impl Actor for StorageMapActor {
    type Context = Context<Self>;
}

impl Handler<message::Create> for StorageMapActor {
    type Result = <message::Create as Message>::Result;

    fn handle(&mut self, msg: message::Create, _ctx: &mut Self::Context) -> Self::Result {
        if self.holder.is_some() {
            return Err(Error::new(ErrorKind::StorageAlreadyExists));
        }

        self.holder = Some(StorageMapActor::create(msg.id, msg.resources)?);
        Ok(self.try_unwrap()?.name().clone())
    }
}

impl Handler<message::Load> for StorageMapActor {
    type Result = <message::Load as Message>::Result;

    fn handle(&mut self, msg: message::Load, _ctx: &mut Self::Context) -> Self::Result {
        if self.holder.is_some() {
            return Err(Error::new(ErrorKind::StorageAlreadyExists));
        }

        self.holder = Some(StorageMapActor::load(&msg.location)?);
        Ok(self.try_unwrap()?.name().clone())
    }
}

impl Handler<message::Save> for StorageMapActor {
    type Result = <message::Save as Message>::Result;

    fn handle(&mut self, msg: message::Save, _ctx: &mut Self::Context) -> Self::Result {
        let map = self.try_unwrap()?;
        serialize_into(map, Path::new(&msg.location))?;
        Ok(())
    }
}

impl Handler<message::ReadChunk> for StorageMapActor {
    type Result = <message::ReadChunk as Message>::Result;

    fn handle(&mut self, msg: message::ReadChunk, _ctx: &mut Self::Context) -> Self::Result {
        let map = self.try_unwrap()?;
        let result = map.read_chunk(msg.chunk)?;
        Ok(result)
    }
}

impl Handler<message::WriteChunk> for StorageMapActor {
    type Result = <message::WriteChunk as Message>::Result;

    fn handle(&mut self, msg: message::WriteChunk, _ctx: &mut Self::Context) -> Self::Result {
        match &mut self.holder {
            Some(ref mut holder) => holder.with_mut(|map| {
                map.write_chunk(msg.chunk, &msg.data)?;
                Ok(())
            }),
            None => Err(Error::new(ErrorKind::StorageDoesNotExist)),
        }?;

        Ok(())
    }
}

impl Handler<message::HasChunk> for StorageMapActor {
    type Result = <message::HasChunk as Message>::Result;

    fn handle(&mut self, msg: message::HasChunk, _ctx: &mut Self::Context) -> Self::Result {
        let map = self.try_unwrap()?;
        Ok(map.has_chunk(msg.chunk))
    }
}

impl Handler<message::HasPiece> for StorageMapActor {
    type Result = <message::HasPiece as Message>::Result;

    fn handle(&mut self, msg: message::HasPiece, _ctx: &mut Self::Context) -> Self::Result {
        let map = self.try_unwrap()?;
        Ok(map.has_piece(msg.piece))
    }
}

impl Handler<message::Prove> for StorageMapActor {
    type Result = <message::Prove as Message>::Result;

    fn handle(&mut self, msg: message::Prove, _ctx: &mut Self::Context) -> Self::Result {
        let map = self.try_unwrap()?;
        Ok(map.prove(msg.leaf_index)?)
    }
}

impl Handler<message::VerifyProof> for StorageMapActor {
    type Result = <message::VerifyProof as Message>::Result;

    fn handle(&mut self, msg: message::VerifyProof, _ctx: &mut Self::Context) -> Self::Result {
        let map = self.try_unwrap()?;
        map.verify(&msg.proof)?;
        Ok(())
    }
}
