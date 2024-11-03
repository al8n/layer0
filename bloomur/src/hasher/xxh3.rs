use super::BloomHasher;

/// A hasher that based on `xxhash_rust::xxh3``.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Xxh3 {
  seed: u64,
}

impl BloomHasher for Xxh3 {
  #[inline]
  fn hash_one(&self, src: &[u8]) -> u32 {
    xxhash_rust::xxh3::xxh3_64_with_seed(src, self.seed) as u32
  }
}

impl Xxh3 {
  /// Creates a new `Xxh3` hasher.
  #[inline]
  pub const fn new() -> Self {
    Self { seed: 0 }
  }

  /// Creates a new `Xxh3` with a seed.
  #[inline]
  pub const fn with_seed(seed: u64) -> Self {
    Self { seed }
  }
}
