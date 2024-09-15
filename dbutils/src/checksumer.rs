/// Checksumer trait.
pub trait Checksumer {
  /// Create a new fresh checksumer internal and calculate the checksum of the buffer without changing the current state.
  /// The result is unrelated to the current state.
  fn checksum(&self, buf: &[u8]) -> u64;

  /// Adds chunk of data to checksum.
  fn update(&mut self, buf: &[u8]);

  /// Resets state to initial state.
  fn reset(&mut self);

  /// Finalize hashing.
  fn digest(&self) -> u64;
}

/// CRC32 checksumer.
#[cfg(feature = "crc32fast")]
#[cfg_attr(docsrs, doc(cfg(feature = "crc32fast")))]
#[derive(Default, Debug, Clone)]
pub struct Crc32(crc32fast::Hasher);

#[cfg(feature = "crc32fast")]
const _: () = {
  impl Crc32 {
    /// Create a new CRC32 checksumer.
    #[inline]
    pub fn new() -> Self {
      Self(crc32fast::Hasher::new())
    }
  }

  impl Checksumer for Crc32 {
    #[inline]
    fn checksum(&self, buf: &[u8]) -> u64 {
      crc32fast::hash(buf) as u64
    }

    #[inline]
    fn update(&mut self, buf: &[u8]) {
      self.0.update(buf)
    }

    #[inline]
    fn reset(&mut self) {
      self.0 = crc32fast::Hasher::new()
    }

    #[inline]
    fn digest(&self) -> u64 {
      self.0.clone().finalize() as u64
    }
  }

  impl super::CheapClone for Crc32 {}
};

/// XxHash checksumer.
#[cfg(feature = "xxhash64")]
#[cfg_attr(docsrs, doc(cfg(feature = "xxhash64")))]
#[derive(Default, Clone)]
pub struct XxHash64 {
  seed: u64,
  hasher: xxhash_rust::xxh64::Xxh64,
}

#[cfg(feature = "xxhash64")]
const _: () = {
  impl XxHash64 {
    /// Create a new XxHash64 with seed 0.
    #[inline]
    pub const fn new() -> Self {
      Self {
        seed: 0,
        hasher: xxhash_rust::xxh64::Xxh64::new(0),
      }
    }

    /// Create a new XxHash64 with a seed.
    #[inline]
    pub const fn with_seed(seed: u64) -> Self {
      Self {
        seed,
        hasher: xxhash_rust::xxh64::Xxh64::new(seed),
      }
    }
  }

  impl Checksumer for XxHash64 {
    #[inline]
    fn checksum(&self, buf: &[u8]) -> u64 {
      xxhash_rust::xxh64::xxh64(buf, self.seed)
    }

    #[inline]
    fn reset(&mut self) {
      self.hasher.reset(self.seed)
    }

    #[inline]
    fn update(&mut self, buf: &[u8]) {
      self.hasher.update(buf)
    }

    #[inline]
    fn digest(&self) -> u64 {
      self.hasher.digest()
    }
  }

  impl super::CheapClone for XxHash64 {}
};

/// XxHash64 (with xxh3 support) checksumer.
#[cfg(feature = "xxhash3")]
#[cfg_attr(docsrs, doc(cfg(feature = "xxhash3")))]
#[derive(Default, Clone)]
pub struct XxHash3 {
  seed: u64,
  hasher: xxhash_rust::xxh3::Xxh3,
}

#[cfg(feature = "xxhash3")]
const _: () = {
  impl XxHash3 {
    /// Create a new XxHash64 with seed 0.
    #[inline]
    pub const fn new() -> Self {
      Self {
        seed: 0,
        hasher: xxhash_rust::xxh3::Xxh3::new(),
      }
    }

    /// Create a new XxHash64 with a seed.
    #[inline]
    pub fn with_seed(seed: u64) -> Self {
      Self {
        seed,
        hasher: xxhash_rust::xxh3::Xxh3::with_seed(seed),
      }
    }
  }

  impl Checksumer for XxHash3 {
    #[inline]
    fn checksum(&self, buf: &[u8]) -> u64 {
      xxhash_rust::xxh3::xxh3_64_with_seed(buf, self.seed)
    }

    #[inline]
    fn update(&mut self, buf: &[u8]) {
      self.hasher.update(buf)
    }

    #[inline]
    fn reset(&mut self) {
      self.hasher.reset()
    }

    #[inline]
    fn digest(&self) -> u64 {
      self.hasher.digest()
    }
  }

  impl super::CheapClone for XxHash3 {}
};
