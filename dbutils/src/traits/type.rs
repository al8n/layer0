mod impls;
use core::cmp::{self, Reverse};

use equivalent::Comparable;
pub use impls::*;

/// Returned when the encoded buffer is too small to hold the bytes format of the [`Type`].
#[derive(Debug)]
pub struct BufferTooSmall {
  required: usize,
  actual: usize,
}

impl BufferTooSmall {
  /// Creates a new instance of the error.
  #[inline]
  const fn new(required: usize, actual: usize) -> Self {
    Self { required, actual }
  }
}

impl core::fmt::Display for BufferTooSmall {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "buffer is too small: required {} bytes, actual {} bytes",
      self.required, self.actual
    )
  }
}

impl core::error::Error for BufferTooSmall {}

/// The type trait for limiting the types that can be used as keys and values.
pub trait Type: core::fmt::Debug {
  /// The reference type for the type.
  type Ref<'a>: TypeRef<'a>;

  /// The error type for encoding the type into a binary format.
  type Error;

  /// Returns the length of the encoded type size.
  fn encoded_len(&self) -> usize;

  /// Encodes the type into a bytes slice, you can assume that the buf length is equal to the value returned by [`encoded_len`](Type::encoded_len).
  ///
  /// Returns the number of bytes written to the buffer.
  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error>;

  /// Encodes the type into a [`Vec<u8>`].
  #[inline]
  #[cfg(any(feature = "alloc", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
  fn encode_into_vec(&self) -> Result<::std::vec::Vec<u8>, Self::Error> {
    let mut buf = ::std::vec![0; self.encoded_len()];
    self.encode(&mut buf)?;
    Ok(buf)
  }
}

impl<T: Type> Type for &T {
  type Ref<'a> = T::Ref<'a>;
  type Error = T::Error;

  #[inline]
  fn encoded_len(&self) -> usize {
    T::encoded_len(*self)
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    T::encode(*self, buf)
  }
}

impl<T: Type> Type for Reverse<T> {
  type Ref<'a> = T::Ref<'a>;
  type Error = T::Error;

  #[inline]
  fn encoded_len(&self) -> usize {
    self.0.encoded_len()
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    self.0.encode(buf)
  }
}

/// The reference type trait for the [`Type`] trait.
pub trait TypeRef<'a>: core::fmt::Debug {
  /// Creates a reference type from a bytes slice.
  ///
  /// ## Safety
  /// - the `src` must the same as the one returned by [`encode`](Type::encode).
  unsafe fn from_slice(src: &'a [u8]) -> Self;
}

/// The key reference trait for comparing `K`.
pub trait KeyRef<'a, K: ?Sized>: Ord + Comparable<K> {
  /// Compares with a type `Q` which can be borrowed from [`K::Ref`](Type::Ref).
  fn compare<Q>(&self, a: &Q) -> cmp::Ordering
  where
    Q: ?Sized + Ord + Comparable<Self>;

  /// Compares two binary formats of the `K` directly.
  ///
  /// ## Safety
  /// - The `a` and `b` must be the same as the one returned by [`K::encode`](Type::encode).
  unsafe fn compare_binary(a: &[u8], b: &[u8]) -> cmp::Ordering;
}
