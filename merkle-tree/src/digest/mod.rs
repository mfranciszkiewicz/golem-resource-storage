pub mod sha512;

use Array;

pub trait Digest {
    fn new() -> Self;
    /// Return output size
    fn output_size() -> usize;
    /// Feed input data
    fn input<A: AsRef<[u8]>>(&mut self, data: A);
    /// Retrieve the result and reset state
    fn result(&mut self) -> Array;
    /// Reset state
    fn reset(&mut self);
}
