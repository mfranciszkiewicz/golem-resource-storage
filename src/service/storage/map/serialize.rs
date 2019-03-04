use std::fs::{create_dir_all, OpenOptions};
use std::path::Path;

use bincode;
use serde::{Deserialize, Serialize};

pub(crate) fn serialize_into<T>(object: &T, path: &Path) -> Result<(), bincode::Error>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .read(false)
        .write(true)
        .truncate(true)
        .open(path)?;

    bincode::serialize_into(&mut file, &object)?;
    Ok(())
}

pub(crate) fn deserialize_from<T>(path: &Path) -> Result<T, bincode::Error>
where
    T: for<'de> Deserialize<'de>,
{
    let file = OpenOptions::new()
        .create(false)
        .read(true)
        .write(false)
        .open(path)?;

    let versioned: T = bincode::deserialize_from(file)?;
    Ok(versioned)
}
