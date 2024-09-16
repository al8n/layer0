/// A trait for creating instances of [`Checksumer`].
///
/// A `BuildChecksumer` is typically used to create
/// [`Checksumer`]s.
///
/// For each instance of `BuildChecksumer`, the [`Checksumer`]s created by
/// [`build_checksumer`] should be identical. That is, if the same stream of bytes
/// is fed into each checksumer, the same output will also be generated.
pub trait BuildChecksumer {
  /// Type of the checksumer that will be created.
  type Checksumer: Checksumer;

  /// Creates a new checksumer.
  ///
  /// Each call to `build_checksumer` on the same instance should produce identical
  /// [`Checksumer`]s.
  ///
  /// # Examples
  ///
  /// ```
  /// use dbutils::checksum::{BuildChecksumer, Crc32};
  ///
  /// let s = Crc32::new();
  /// let new_s = s.build_checksumer();
  /// ```
  fn build_checksumer(&self) -> Self::Checksumer;

  /// Calculates the checksum of a byte slice.
  fn checksum_one(&self, src: &[u8]) -> u64;
}

/// Checksumer trait.
pub trait Checksumer {
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

  impl BuildChecksumer for Crc32 {
    type Checksumer = Self;

    #[inline]
    fn build_checksumer(&self) -> Self::Checksumer {
      Self::new()
    }

    #[inline]
    fn checksum_one(&self, src: &[u8]) -> u64 {
      crc32fast::hash(src) as u64
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

  impl BuildChecksumer for XxHash64 {
    type Checksumer = Self;

    #[inline]
    fn build_checksumer(&self) -> Self::Checksumer {
      Self::with_seed(self.seed)
    }

    #[inline]
    fn checksum_one(&self, src: &[u8]) -> u64 {
      xxhash_rust::xxh64::xxh64(src, self.seed)
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

  impl BuildChecksumer for XxHash3 {
    type Checksumer = Self;

    #[inline]
    fn build_checksumer(&self) -> Self::Checksumer {
      Self::with_seed(self.seed)
    }

    #[inline]
    fn checksum_one(&self, src: &[u8]) -> u64 {
      xxhash_rust::xxh3::xxh3_64_with_seed(src, self.seed)
    }
  }

  impl super::CheapClone for XxHash3 {}
};
