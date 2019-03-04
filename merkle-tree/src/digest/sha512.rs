use digest::Digest;
use ring;
use ring::digest::{SHA512, SHA512_OUTPUT_LEN};

use Array;

#[derive(Clone)]
pub struct Sha512 {
    ctx: Option<ring::digest::Context>,
}

impl Sha512 {
    #[inline]
    fn new_ctx() -> ring::digest::Context {
        ring::digest::Context::new(&SHA512)
    }
}

impl Digest for Sha512 {
    fn new() -> Self {
        Sha512 {
            ctx: Some(Self::new_ctx()),
        }
    }

    #[inline(always)]
    fn output_size() -> usize {
        SHA512_OUTPUT_LEN
    }

    #[inline]
    fn input<A: AsRef<[u8]>>(&mut self, data: A) {
        let ctx = self.ctx.as_mut().unwrap();
        ctx.update(data.as_ref());
    }

    fn result(&mut self) -> Array {
        let ctx = self.ctx.take().unwrap();
        let digest = ctx.finish();

        self.reset();
        digest.as_ref().to_vec()
    }

    #[inline]
    fn reset(&mut self) {
        self.ctx = Some(Self::new_ctx());
    }
}
