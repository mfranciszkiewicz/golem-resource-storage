#[macro_use]
pub mod resource;

use std::fmt;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::ops::DerefMut;

use indexmap::IndexMap;
use serde::de;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use self::resource::GenericResourcePtr;
use storage::error::ErrorKind;
use storage::resource::{Resource, ResourcePtr};
use storage::shard::{Shard, ShardReader, ShardWriter, Sharded};
use storage::view::uniform::UniformView;
use storage::view::{View, ViewVec};
use storage::{Result, Size, Storage, StorageId};

#[derive(Serialize, Deserialize)]
pub struct GenericStorage<R>
where
    R: Resource,
{
    pub name: StorageId,
    #[serde(serialize_with = "serialize_resources")]
    #[serde(deserialize_with = "deserialize_resources")]
    resources: IndexMap<StorageId, <GenericStorage<R> as Storage>::Ptr>,
    total_size: usize,
}

impl<R> GenericStorage<R>
where
    R: Resource,
{
    pub fn collect<S, I>(items: I) -> Result<Vec<(String, usize)>>
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        let mut results = Vec::new();

        for location in items.into_iter() {
            let location: String = location.into();
            let meta = R::metadata(&location)?;
            results.push((location, meta.size()));
        }

        Ok(results)
    }

    fn add(&mut self, location: &String, size: &usize) -> Result<()> {
        let resource = if R::exists(location) {
            R::open(location)?
        } else {
            R::create(location, size)?
        };

        if resource.size() != *size {
            return err_new!(ErrorKind::SizeMismatch(resource.size(), *size));
        }

        self.resources
            .insert(location.clone(), ResourcePtr::new(resource));
        Ok(())
    }
}

impl<R> Size for GenericStorage<R>
where
    R: Resource,
{
    #[inline(always)]
    fn size(&self) -> usize {
        self.total_size
    }
}

impl<R> Storage for GenericStorage<R>
where
    R: Resource,
{
    type Ptr = GenericResourcePtr<R>;

    fn new(name: StorageId, items: Vec<(String, usize)>) -> Result<Self> {
        let mut storage = GenericStorage {
            name,
            resources: IndexMap::new(),
            total_size: 0,
        };

        items.iter().try_for_each(|(location, size)| {
            storage.total_size += size;
            storage.add(location, size)
        })?;

        Ok(storage)
    }

    fn read(&self, offset: usize, into: &mut [u8]) -> Result<usize> {
        let view = self.view(offset, into.len())?;

        let mut start: usize = 0;
        let mut end: usize;

        for (mut resource, shard) in view {
            let mut borrowed = resource.try_borrow_mut()?;
            let resource = borrowed.deref_mut();

            end = start + shard.size();
            let slice = &mut into[start..end];
            start += self.read_shard(resource, &shard, slice)?;
        }

        Ok(start)
    }

    fn write(&self, offset: usize, from: &[u8]) -> Result<usize> {
        let view = self.view(offset, from.len())?;

        let mut start: usize = 0;
        let mut end: usize;
        let mut slice: &[u8];

        for (mut resource, shard) in view {
            let mut borrowed = resource.try_borrow_mut()?;
            let resource = borrowed.deref_mut();

            end = start + shard.size();
            slice = &from[start..end];
            start += self.write_shard(resource, &shard, slice)?;
        }

        Ok(start)
    }

    fn name(&self) -> &StorageId {
        &self.name
    }
}

impl<R> Sharded for GenericStorage<R>
where
    R: Resource,
{
    fn view(&self, start_idx: usize, size: usize) -> Result<ViewVec<<Self as Storage>::Ptr>> {
        let mut builder = UniformView::<<Self as Storage>::Ptr>::new(start_idx, start_idx + size);
        self.resources
            .values()
            .all(|resource| builder.add(resource));
        builder.build()
    }
}

impl<R> ShardReader for GenericStorage<R>
where
    R: Resource,
{
    #[inline]
    fn read_shard(
        &self,
        resource: &mut <<Self as Storage>::Ptr as ResourcePtr>::Target,
        shard: &Shard,
        into: &mut [u8],
    ) -> Result<usize> {
        self.seek(resource, shard)?;
        let read = resource.handle().read(into)?;
        Ok(read)
    }
}

impl<R> ShardWriter for GenericStorage<R>
where
    R: Resource,
{
    #[inline]
    fn write_shard(
        &self,
        resource: &mut <<Self as Storage>::Ptr as ResourcePtr>::Target,
        shard: &Shard,
        from: &[u8],
    ) -> Result<usize> {
        self.seek(resource, shard)?;
        let written = resource.handle().write(from)?;
        Ok(written)
    }
}

struct ResourceSeqVisitor<P> {
    phantom: PhantomData<P>,
}

impl<'de, 'a, P: 'a> de::Visitor<'de> for ResourceSeqVisitor<P>
where
    P: ResourcePtr,
{
    type Value = IndexMap<StorageId, P>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut map = Self::Value::new();

        while let Some(elem) = seq.next_element()? {
            match P::Target::open(&elem) {
                Ok(res) => {
                    let ptr = P::new(res);
                    map.insert(elem, ptr);
                }
                Err(err) => {
                    return Err(de::Error::custom(err));
                }
            }
        }

        Ok(map)
    }
}

fn serialize_resources<S, P>(
    map: &IndexMap<String, P>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    P: ResourcePtr,
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(map.len()))?;
    for key in map.keys() {
        seq.serialize_element(key)?;
    }
    seq.end()
}

fn deserialize_resources<'de, D, P>(
    deserializer: D,
) -> std::result::Result<IndexMap<StorageId, P>, D::Error>
where
    P: ResourcePtr,
    D: Deserializer<'de>,
{
    let visitor = ResourceSeqVisitor {
        phantom: PhantomData,
    };
    deserializer.deserialize_seq(visitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use storage::tests::common::resource::TestResource;
    use streaming_iterator::StreamingIterator;

    type TestStorage = GenericStorage<TestResource>;

    fn make_size(n: i32) -> i32 {
        (n + 1) * 128
    }

    fn make_vec(size: usize) -> Vec<u8> {
        (0..size).map(|n| n as u8).collect()
    }

    fn resources(count: usize) -> Vec<(String, usize)> {
        (0..count)
            .map(|n| {
                let location: String = format!("location_{}", n);
                let size: usize = make_size(n as i32) as usize;
                (location, size)
            })
            .collect()
    }

    fn resources_of_size(count: usize, size: usize) -> Vec<(String, usize)> {
        (0..count)
            .map(|n| (format!("location_{}", n).to_string(), size))
            .collect()
    }

    #[test]
    fn test_new() {
        let resources = resources(10);
        let sizes: Vec<usize> = resources.iter().map(|(_, size)| *size).collect();
        let storage = TestStorage::new("Test storage".to_string(), resources).unwrap();

        assert_eq!(storage.name, "Test storage".to_string());
        assert_eq!(storage.total_size, sizes.iter().sum::<usize>());
        assert_eq!(storage.resources.len(), 10);
    }

    #[test]
    fn test_read() {
        let resources = resources(10);
        let sizes: Vec<usize> = resources.iter().map(|(_, size)| *size).collect();
        let storage = TestStorage::new("Test storage".to_string(), resources).unwrap();

        let to_read = 1024 as usize;
        let size_last = to_read - sizes[0..3].iter().sum::<usize>();

        let mut expected: Vec<u8> = make_vec(sizes[0]);
        expected.extend(make_vec(sizes[1]));
        expected.extend(make_vec(sizes[2]));
        expected.extend(make_vec(size_last));

        let mut read = vec![0 as u8; to_read];
        storage.read(0 as usize, &mut read[..]).unwrap();

        assert_eq!(read.len(), expected.len());
        assert_eq!(read[..], expected[..]);
    }

    #[test]
    fn test_read_0() {
        let resources = resources(1);
        let storage = TestStorage::new("Test storage".to_string(), resources).unwrap();

        let mut read = vec![0 as u8; 0];
        match storage.read(0 as usize, &mut read[..]) {
            Ok(_) => (),
            Err(_) => panic!("Read failed"),
        }
    }

    #[test]
    fn test_read_from_empty() {
        let resources = resources(0);
        let storage = TestStorage::new("Test storage".to_string(), resources).unwrap();

        let mut read = vec![0 as u8; 1];
        match storage.read(0 as usize, &mut read[..]) {
            Ok(_) => panic!("Reading out of bounds was possible"),
            Err(_) => (),
        }
    }

    #[test]
    fn test_read_out_of_bounds() {
        let resources = resources(2);
        let storage = TestStorage::new("Test storage".to_string(), resources).unwrap();

        let mut read = vec![0 as u8; 1024];
        match storage.read(0 as usize, &mut read[..]) {
            Ok(_) => panic!("Reading out of bounds was possible"),
            Err(_) => (),
        }
    }

    #[test]
    fn test_write() {
        let resources = resources(8);
        let storage = TestStorage::new("Test storage".to_string(), resources).unwrap();

        let expected: Vec<u8> = vec![1 as u8; 1024];
        storage.write(0 as usize, &expected[..]).unwrap();

        let mut read = vec![0 as u8; 1024];
        storage.read(0 as usize, &mut read[..]).unwrap();

        assert_eq!(read.len(), expected.len());
        assert_eq!(read[..], expected[..]);
    }

    #[test]
    fn test_write_with_offset() {
        let resources = resources(128);
        let storage = TestStorage::new("Test storage".to_string(), resources).unwrap();

        let mut read = vec![0 as u8; 1024];
        let mut expected = vec![1 as u8; 3072];
        storage.write(0 as usize, &expected[..]).unwrap();

        expected = vec![2 as u8; 1024];
        storage.write(1024 as usize, &expected[..]).unwrap();
        storage.read(1024 as usize, &mut read[..]).unwrap();
        assert_eq!(read[..], expected[..]);

        expected = vec![1 as u8; 1024];
        storage.read(0 as usize, &mut read[..]).unwrap();
        assert_eq!(read[..], expected[..]);
        storage.read(2048 as usize, &mut read[..]).unwrap();
        assert_eq!(read[..], expected[..]);
    }

    #[test]
    fn test_write_0() {
        let resources = resources(1);
        let storage = TestStorage::new("Test storage".to_string(), resources).unwrap();

        let expected: Vec<u8> = make_vec(0);
        match storage.write(0 as usize, &expected[..]) {
            Ok(_) => (),
            Err(_) => panic!("Write failed"),
        }
    }

    #[test]
    fn test_write_to_empty() {
        let resources = resources(0);
        let storage = TestStorage::new("Test storage".to_string(), resources).unwrap();

        let expected: Vec<u8> = make_vec(1);
        match storage.write(0 as usize, &expected[..]) {
            Ok(_) => panic!("Writing out of bounds was possible"),
            Err(_) => (),
        }
    }

    #[test]
    fn test_write_out_of_bounds() {
        let resources = resources(2);
        let storage = TestStorage::new("Test storage".to_string(), resources).unwrap();

        let expected = make_vec(1024);
        match storage.write(0 as usize, &expected[..]) {
            Ok(_) => panic!("Writing out of bounds was possible"),
            Err(_) => (),
        }
    }

    #[test]
    fn test_iter() {
        let resources = resources_of_size(100, 128);
        let storage = TestStorage::new("Test storage".to_string(), resources).unwrap();
        let mut iter = storage.iter(256);

        assert_eq!(iter.size_hint().1.unwrap(), storage.total_size / 256);

        let mut expected = make_vec(128);
        expected.extend(make_vec(128));

        while let Some(data) = iter.next() {
            assert_eq!(data[..], expected[..]);
        }
    }
}
