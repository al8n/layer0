mod simmurur;
pub use simmurur::*;

/// [`Xxh32`](xxhash_rust::xxh32::Xxh32) hasher.
#[cfg(feature = "xxhash32")]
#[cfg_attr(docsrs, doc(cfg(feature = "xxhash32")))]
pub mod xxh32;
#[cfg(feature = "xxhash32")]
#[cfg_attr(docsrs, doc(cfg(feature = "xxhash32")))]
pub use xxh32::Xxh32;

/// [`Xxhash3`](xxhash_rust::xxh3::Xxh3) hasher.
#[cfg(feature = "xxhash3")]
#[cfg_attr(docsrs, doc(cfg(feature = "xxhash3")))]
pub mod xxh3;
#[cfg(feature = "xxhash3")]
#[cfg_attr(docsrs, doc(cfg(feature = "xxhash3")))]
pub use xxh3::Xxh3;

/// A trait for hashing keys.
pub trait BloomHasher {
  /// Hashes the key and returns the hash value.
  fn hash_one(&self, src: &[u8]) -> u32;
}
