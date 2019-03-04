use serde::{Deserialize, Serialize};

use service::error::Error;
use storage::file::resource;
use storage::generic::GenericStorage;
use storage::map::StorageMap;

pub type StorageV1 = GenericStorage<resource::FileResource>;
pub type StorageMapV1 = StorageMap<StorageV1>;
pub type StorageMapVersion = StorageMapV1;

#[derive(Serialize, Deserialize)]
pub enum VersionedStorageMap {
    V1(StorageMapV1),
}

impl VersionedStorageMap {
    pub const DEFAULT: fn(StorageMapVersion) -> VersionedStorageMap = VersionedStorageMap::V1;
}

impl VersionedStorageMap {
    pub fn wrap(storage: StorageMapVersion) -> Self {
        VersionedStorageMap::DEFAULT(storage)
    }

    pub fn try_unwrap(&self) -> Result<&StorageMapVersion, Error> {
        match self {
            VersionedStorageMap::V1(map) => Ok(&map),
        }
    }

    pub fn with_mut<R, F>(&mut self, handler: F) -> Result<R, Error>
    where
        F: Fn(&mut StorageMapVersion) -> Result<R, Error>,
    {
        match self {
            VersionedStorageMap::V1(ref mut map) => handler(map),
        }
    }
}
