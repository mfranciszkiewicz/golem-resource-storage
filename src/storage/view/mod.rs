pub mod uniform;

use storage::resource::ResourcePtr;
use storage::shard::Shard;
use storage::Result;

pub type ViewVec<R> = Vec<(R, Shard)>;

pub trait View<P>
where
    P: ResourcePtr,
{
    fn add(&mut self, ptr: &P) -> bool;
    fn build(self) -> Result<ViewVec<P>>;
}
