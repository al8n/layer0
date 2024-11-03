use super::BloomHasher;

/// A hasher that based on `xxhash::xxh32`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg(feature = "xxhash32")]
#[cfg_attr(docsrs, doc(cfg(feature = "xxhash32")))]
pub struct Xxh32 {
  seed: u32,
}

#[cfg(feature = "xxhash32")]
const _: () = {
  impl BloomHasher for Xxh32 {
    #[inline]
    fn hash_one(&self, src: &[u8]) -> u32 {
      xxhash_rust::xxh32::xxh32(src, self.seed)
    }
  }

  impl Xxh32 {
    /// Creates a new `Xxh32` hasher.
    #[inline]
    pub const fn new() -> Self {
      Self { seed: 0 }
    }

    /// Creates a new `Xxh32` with a seed.
    #[inline]
    pub const fn with_seed(seed: u32) -> Self {
      Self { seed }
    }
  }
};
