//! Utils for developing database
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

pub use cheap_clone::CheapClone;

use core::{
  cmp,
  ops::{Bound, RangeBounds},
};

/// Comparator is used for key-value database developers to define their own key comparison logic.
/// e.g. some key-value database developers may want to alpabetically comparation
pub trait Comparator: core::fmt::Debug {
  /// Compares two byte slices.
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering;

  /// Returns if a is contained in range.
  fn contains(&self, start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool;
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<C: Comparator> Comparator for std::sync::Arc<C> {
  #[inline]
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering {
    (**self).compare(a, b)
  }

  fn contains(&self, start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool {
    (**self).contains(start_bound, end_bound, key)
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<C: Comparator> Comparator for std::rc::Rc<C> {
  #[inline]
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering {
    (**self).compare(a, b)
  }

  fn contains(&self, start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool {
    (**self).contains(start_bound, end_bound, key)
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<C: Comparator> Comparator for std::boxed::Box<C> {
  #[inline]
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering {
    (**self).compare(a, b)
  }

  fn contains(&self, start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool {
    (**self).contains(start_bound, end_bound, key)
  }
}

/// Ascend is a comparator that compares byte slices in ascending order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ascend;

impl Comparator for Ascend {
  #[inline]
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering {
    a.cmp(b)
  }

  #[inline]
  fn contains(&self, start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool {
    <(Bound<&[u8]>, Bound<&[u8]>) as RangeBounds<&[u8]>>::contains::<&[u8]>(
      &(start_bound, end_bound),
      &key,
    )
  }
}

/// Descend is a comparator that compares byte slices in descending order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Descend;

impl Comparator for Descend {
  #[inline]
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering {
    b.cmp(a)
  }

  #[inline]
  fn contains(&self, start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool {
    <(Bound<&[u8]>, Bound<&[u8]>) as RangeBounds<&[u8]>>::contains::<&[u8]>(
      &(start_bound, end_bound),
      &key,
    )
  }
}

/// Checksumer trait.
pub trait Checksumer {
  /// Calculate the checksum of the buffer.
  fn checksum(&self, buf: &[u8]) -> u64;
}

/// CRC32 checksumer.
#[cfg(feature = "crc32fast")]
#[cfg_attr(docsrs, doc(cfg(feature = "crc32fast")))]
#[derive(Default, Debug, Copy, Clone)]
pub struct Crc32;

#[cfg(feature = "crc32fast")]
impl Checksumer for Crc32 {
  #[inline]
  fn checksum(&self, buf: &[u8]) -> u64 {
    crc32fast::hash(buf) as u64
  }
}

/// XxHash checksumer.
#[cfg(feature = "xxhash64")]
#[cfg_attr(docsrs, doc(cfg(feature = "xxhash64")))]
#[derive(Default, Debug, Copy, Clone)]
pub struct XxHash64(u64);

#[cfg(feature = "xxhash64")]
impl XxHash64 {
  /// Create a new XxHash64 with seed 0.
  #[inline]
  pub const fn new() -> Self {
    Self(0)
  }

  /// Create a new XxHash64 with a seed.
  #[inline]
  pub const fn with_seed(seed: u64) -> Self {
    Self(seed)
  }
}

#[cfg(feature = "xxhash64")]
impl Checksumer for XxHash64 {
  #[inline]
  fn checksum(&self, buf: &[u8]) -> u64 {
    xxhash_rust::xxh64::xxh64(buf, self.0)
  }
}

/// XxHash64 (with xxh3 support) checksumer.
#[cfg(feature = "xxhash3")]
#[cfg_attr(docsrs, doc(cfg(feature = "xxhash3")))]
#[derive(Default, Debug, Copy, Clone)]
pub struct XxHash3(u64);

#[cfg(feature = "xxhash3")]
impl XxHash3 {
  /// Create a new XxHash64 with seed 0.
  #[inline]
  pub const fn new() -> Self {
    Self(0)
  }

  /// Create a new XxHash64 with a seed.
  #[inline]
  pub const fn with_seed(seed: u64) -> Self {
    Self(seed)
  }
}

#[cfg(feature = "xxhash3")]
impl Checksumer for XxHash3 {
  #[inline]
  fn checksum(&self, buf: &[u8]) -> u64 {
    xxhash_rust::xxh3::xxh3_64_with_seed(buf, self.0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_desc() {
    let desc = Descend;
    assert_eq!(desc.compare(b"abc", b"def"), cmp::Ordering::Greater);
    assert_eq!(desc.compare(b"def", b"abc"), cmp::Ordering::Less);
    assert_eq!(desc.compare(b"abc", b"abc"), cmp::Ordering::Equal);

    assert!(desc.contains(Bound::Included(b"a"), Bound::Excluded(b"d"), b"b"));
  }

  #[test]
  fn test_asc() {
    let asc = Ascend;
    assert_eq!(asc.compare(b"abc", b"def"), cmp::Ordering::Less);
    assert_eq!(asc.compare(b"def", b"abc"), cmp::Ordering::Greater);
    assert_eq!(asc.compare(b"abc", b"abc"), cmp::Ordering::Equal);

    assert!(asc.contains(Bound::Included(b"a"), Bound::Excluded(b"d"), b"b"));
  }
}
