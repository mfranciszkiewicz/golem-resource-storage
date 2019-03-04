use std::cmp::min;

use storage::error::ErrorKind;
use storage::resource::ResourcePtr;
use storage::shard::Shard;
use storage::view::{View, ViewVec};
use storage::{Result, Size};

#[derive(Debug)]
pub struct UniformView<P>
where
    P: ResourcePtr,
{
    /// Input start index
    start: usize,
    /// Input end index
    end: usize,
    /// Local offset
    offset: usize,
    /// Bytes consumed
    consumed: usize,
    /// Resulting view vector
    view: ViewVec<P>,
}

impl<P> UniformView<P>
where
    P: ResourcePtr,
{
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            offset: 0,
            consumed: 0,
            view: Vec::new(),
        }
    }
}

impl<P> View<P> for UniformView<P>
where
    P: ResourcePtr,
{
    fn add(&mut self, pointer: &P) -> bool {
        let size = pointer.size();
        let offset = self.offset;

        self.offset += size;

        if offset >= self.end {
            return false;
        }
        if size == 0 || self.offset < self.start {
            return true;
        }

        let start = self.start + self.consumed - offset;
        let consumed = min(size, self.size() - self.consumed) - start;

        let pointer = P::clone(pointer);
        let shard = Shard {
            start,
            end: start + consumed,
        };

        self.view.push((pointer, shard));
        self.consumed += consumed;
        self.consumed != self.size()
    }

    fn build(self) -> Result<ViewVec<P>> {
        if self.consumed != self.size() {
            return err_new!(ErrorKind::ViewBuildError(self.start, self.end, self.offset));
        }

        Ok(self.view)
    }
}

impl<P> Size for UniformView<P>
where
    P: ResourcePtr,
{
    #[inline]
    fn size(&self) -> usize {
        self.end - self.start
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use storage::resource::Resource;
    use storage::shard::Shard;
    use storage::tests::common::resource::TestResource;

    type Ptr = Rc<RefCell<TestResource>>;

    macro_rules! new_resource {
        ($num:expr, $size:expr) => {{
            let result = TestResource::create(&format!("location_{}", $num), &($size as usize));
            Rc::new(RefCell::new(result.unwrap()))
        }};
    }

    fn resources() -> Vec<Ptr> {
        vec![
            new_resource!(0, 1024),
            new_resource!(1, 0),
            new_resource!(2, 511),
            new_resource!(3, 257),
            new_resource!(4, 0),
            new_resource!(5, 64),
            new_resource!(6, 128),
            new_resource!(7, 64),
        ]
    }

    fn locations() -> Vec<String> {
        [0, 2, 3, 5, 6, 7]
            .iter()
            .map(|n| format!("location_{}", n).to_string())
            .collect()
    }

    #[test]
    fn test_build_failure() {
        let mut uniform = UniformView::<Ptr>::new(0, 2049);
        resources().iter().all(|resource| uniform.add(resource));

        if let Ok(_) = uniform.build() {
            panic!("The view should not have been built")
        }
    }

    #[test]
    fn test_build() {
        let mut uniform = UniformView::<Ptr>::new(1, 2047);
        resources().iter().all(|resource| uniform.add(resource));

        let shards = vec![
            Shard {
                start: 1,
                end: 1024,
            },
            Shard { start: 0, end: 511 },
            Shard { start: 0, end: 257 },
            Shard { start: 0, end: 64 },
            Shard { start: 0, end: 128 },
            Shard { start: 0, end: 63 },
        ];

        let locations = locations();
        let view = uniform.build().unwrap();
        assert_eq!(view.len(), shards.len());
        assert_eq!(view.len(), locations.len());

        for i in 0..view.len() {
            let (resource, shard) = &view[i];
            assert_eq!(*resource.borrow().location(), locations[i]);
            assert_eq!(shard, &shards[i]);
        }
    }
}
