use core::cmp::{self, Reverse};

use either::Either;
pub use impls::*;

use crate::{buffer::VacantBuffer, equivalent::*};

mod impls;
mod lazy_ref;

pub use lazy_ref::LazyRef;

/// The type trait for limiting the types that can be used as keys and values.
pub trait Type: core::fmt::Debug {
  /// The reference type for the type.
  type Ref<'a>: TypeRef<'a>;

  /// The error type for encoding the type into a binary format.
  type Error;

  /// Returns the length of the encoded type size.
  fn encoded_len(&self) -> usize;

  /// Encodes the type into a bytes slice.
  ///
  /// Returns the number of bytes written to the buffer.
  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    self.encode_to_buffer(&mut VacantBuffer::from(buf))
  }

  /// Encodes the type into a [`VacantBuffer`].
  ///
  /// Returns the number of bytes written to the buffer.
  fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error>;

  /// Encodes the type into a [`Vec<u8>`].
  #[inline]
  #[cfg(any(feature = "alloc", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
  fn encode_into_vec(&self) -> Result<::std::vec::Vec<u8>, Self::Error> {
    let mut buf = ::std::vec![0; self.encoded_len()];
    self.encode(&mut buf)?;
    Ok(buf)
  }

  /// Returns the bytes format of the type, which should be the same as the one returned by [`encode`](Type::encode).
  ///
  /// This method is used for some types like `[u8]`, `str` can be directly converted into the bytes format.
  #[inline]
  fn as_encoded(&self) -> Option<&[u8]> {
    None
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

  #[inline]
  fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    T::encode_to_buffer(self, buf)
  }

  #[inline]
  fn as_encoded(&self) -> Option<&[u8]> {
    T::as_encoded(*self)
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

  #[inline]
  fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    self.0.encode_to_buffer(buf)
  }

  #[inline]
  fn as_encoded(&self) -> Option<&[u8]> {
    self.0.as_encoded()
  }
}

/// The reference type trait for the [`Type`] trait.
pub trait TypeRef<'a>: core::fmt::Debug + Copy + Sized {
  /// Creates a reference type from a bytes slice.
  ///
  /// ## Safety
  /// - the `src` must the same as the one returned by [`encode`](Type::encode).
  unsafe fn from_slice(src: &'a [u8]) -> Self;

  /// Returns the original bytes slice of the reference type.
  ///
  /// This method can return `None` if your reference type does not keep the original bytes slice.
  #[inline]
  fn as_raw(&self) -> Option<&'a [u8]> {
    None
  }
}

/// A wrapper around a generic type that can be used to construct for insertion.
#[repr(transparent)]
#[derive(Debug)]
pub struct MaybeStructured<'a, T: ?Sized> {
  data: Either<&'a T, &'a [u8]>,
}

impl<T: ?Sized> Clone for MaybeStructured<'_, T> {
  #[inline]
  fn clone(&self) -> Self {
    *self
  }
}

impl<T: ?Sized> Copy for MaybeStructured<'_, T> {}

impl<'a, T: 'a> PartialEq<T> for MaybeStructured<'a, T>
where
  T: ?Sized + PartialEq + Type + for<'b> Equivalent<T::Ref<'b>>,
{
  #[inline]
  fn eq(&self, other: &T) -> bool {
    match &self.data {
      Either::Left(val) => (*val).eq(other),
      Either::Right(val) => {
        let ref_ = unsafe { <T::Ref<'_> as TypeRef<'_>>::from_slice(val) };
        other.equivalent(&ref_)
      }
    }
  }
}

impl<'a, T: 'a> PartialEq for MaybeStructured<'a, T>
where
  T::Ref<'a>: Equivalent<T>,
  T: ?Sized + PartialEq + Type,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    match (&self.data, &other.data) {
      (Either::Left(val), Either::Left(other_val)) => val.eq(other_val),
      (Either::Right(val), Either::Right(other_val)) => val.eq(other_val),
      (Either::Left(val), Either::Right(other_val)) => {
        let ref_ = unsafe { <T::Ref<'_> as TypeRef<'_>>::from_slice(other_val) };
        ref_.equivalent(val)
      }
      (Either::Right(val), Either::Left(other_val)) => {
        let ref_ = unsafe { <T::Ref<'_> as TypeRef<'_>>::from_slice(val) };
        ref_.equivalent(other_val)
      }
    }
  }
}

impl<'a, T: 'a> Eq for MaybeStructured<'a, T>
where
  T::Ref<'a>: Equivalent<T>,
  T: ?Sized + Eq + Type,
{
}

impl<'a, T: 'a> PartialOrd for MaybeStructured<'a, T>
where
  T: ?Sized + Ord + Type + for<'b> Comparable<T::Ref<'b>>,
  for<'b> T::Ref<'b>: Comparable<T> + Ord,
{
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl<'a, T: 'a> PartialOrd<T> for MaybeStructured<'a, T>
where
  T: ?Sized + PartialOrd + Type + for<'b> Comparable<T::Ref<'b>>,
{
  #[inline]
  fn partial_cmp(&self, other: &T) -> Option<core::cmp::Ordering> {
    match &self.data {
      Either::Left(val) => (*val).partial_cmp(other),
      Either::Right(val) => {
        let ref_ = unsafe { <T::Ref<'_> as TypeRef<'_>>::from_slice(val) };
        Some(other.compare(&ref_).reverse())
      }
    }
  }
}

impl<'a, T: 'a> Ord for MaybeStructured<'a, T>
where
  T: ?Sized + Ord + Type + for<'b> Comparable<T::Ref<'b>>,
  for<'b> T::Ref<'b>: Comparable<T> + Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    match (&self.data, &other.data) {
      (Either::Left(val), Either::Left(other_val)) => (*val).cmp(other_val),
      (Either::Right(val), Either::Right(other_val)) => {
        let this = unsafe { <T::Ref<'_> as TypeRef<'_>>::from_slice(val) };
        let other = unsafe { <T::Ref<'_> as TypeRef<'_>>::from_slice(other_val) };
        this.cmp(&other)
      }
      (Either::Left(val), Either::Right(other_val)) => {
        let other = unsafe { <T::Ref<'_> as TypeRef<'_>>::from_slice(other_val) };
        other.compare(*val).reverse()
      }
      (Either::Right(val), Either::Left(other_val)) => {
        let this = unsafe { <T::Ref<'_> as TypeRef<'_>>::from_slice(val) };
        this.compare(*other_val)
      }
    }
  }
}

impl<'a, T: 'a + Type + ?Sized> MaybeStructured<'a, T> {
  /// Returns the encoded length.
  #[inline]
  pub fn encoded_len(&self) -> usize {
    match &self.data {
      Either::Left(val) => val.encoded_len(),
      Either::Right(val) => val.len(),
    }
  }

  /// Encodes the generic into the buffer.
  ///
  /// ## Panics
  /// - if the buffer is not large enough.
  #[inline]
  pub fn encode(&self, buf: &mut [u8]) -> Result<usize, T::Error> {
    match &self.data {
      Either::Left(val) => val.encode(buf),
      Either::Right(val) => {
        buf.copy_from_slice(val);
        Ok(buf.len())
      }
    }
  }

  /// Encodes the generic into the given buffer.
  ///
  /// ## Panics
  /// - if the buffer is not large enough.
  #[inline]
  pub fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, T::Error> {
    match &self.data {
      Either::Left(val) => val.encode_to_buffer(buf),
      Either::Right(val) => {
        buf.put_slice_unchecked(val);
        Ok(buf.len())
      }
    }
  }
}

impl<'a, T: 'a + ?Sized> MaybeStructured<'a, T> {
  /// Returns the value contained in the generic.
  #[inline]
  pub const fn data(&self) -> Either<&'a T, &'a [u8]> {
    self.data
  }

  /// Creates a new unstructured `MaybeStructured` from bytes for querying or inserting.
  ///
  /// ## Safety
  /// - the `slice` must the same as the one returned by [`T::encode`](Type::encode).
  #[inline]
  pub const unsafe fn from_slice(slice: &'a [u8]) -> Self {
    Self {
      data: Either::Right(slice),
    }
  }
}

impl<'a, T: 'a + ?Sized> From<&'a T> for MaybeStructured<'a, T> {
  #[inline]
  fn from(value: &'a T) -> Self {
    Self {
      data: Either::Left(value),
    }
  }
}
