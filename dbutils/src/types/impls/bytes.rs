use ::equivalent::*;
use core::{borrow::Borrow, cmp::Ordering};

use super::*;

macro_rules! impls {
  ($( $(#[cfg($cfg:meta)])? $ty:ty),+ $(,)?) => {
    $(
      $(#[cfg($cfg)])?
      const _: () = {
        impl Type for $ty {
          type Ref<'a> = SliceRef<'a>;
          type Error = InsufficientBuffer;

          #[inline]
          fn encoded_len(&self) -> usize {
            self.len()
          }

          #[inline]
          fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
            buf.put_slice(self.as_ref())
          }

          #[inline]
          fn as_encoded(&self) -> Option<&[u8]> {
            Some(self.as_ref())
          }
        }

        impl_cmp! {
          SliceRef(&[u8])
          @(bool) PartialEq::eq($ty, &$ty),
          @(bool) Equivalent::equivalent($ty, &$ty),
          @(Ordering) Comparable::compare($ty, &$ty),
          @(Option<Ordering>) PartialOrd::partial_cmp($ty, &$ty),
        }
      };
    )*
  };
}

impl<'a> TypeRef<'a> for &'a [u8] {
  unsafe fn from_slice(src: &'a [u8]) -> Self {
    src
  }

  #[inline]
  fn as_raw(&self) -> Option<&'a [u8]> {
    Some(self)
  }
}

/// A wrapper type for `&'a [u8]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SliceRef<'a>(&'a [u8]);

impl<'a> SliceRef<'a> {
  /// Returns the inner bytes slice.
  #[inline]
  pub const fn as_bytes(&self) -> &'a [u8] {
    self.0
  }
}

impl Borrow<[u8]> for SliceRef<'_> {
  #[inline]
  fn borrow(&self) -> &[u8] {
    self.0
  }
}

impl<'a> From<&'a [u8]> for SliceRef<'a> {
  #[inline]
  fn from(src: &'a [u8]) -> Self {
    Self(src)
  }
}

impl<'a> From<SliceRef<'a>> for &'a [u8] {
  #[inline]
  fn from(src: SliceRef<'a>) -> Self {
    src.0
  }
}

impl<'a> TypeRef<'a> for SliceRef<'a> {
  #[inline]
  unsafe fn from_slice(src: &'a [u8]) -> Self {
    Self(src)
  }
}

impl AsRef<[u8]> for SliceRef<'_> {
  #[inline]
  fn as_ref(&self) -> &[u8] {
    self.0
  }
}

impl core::ops::Deref for SliceRef<'_> {
  type Target = [u8];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.0
  }
}

impl<'a, K> KeyRef<'a, K> for SliceRef<'a>
where
  K: ?Sized + Type<Ref<'a> = SliceRef<'a>>,
  SliceRef<'a>: Comparable<K>,
{
  #[inline]
  fn compare<Q>(&self, a: &Q) -> core::cmp::Ordering
  where
    Q: ?Sized + Ord + Comparable<Self>,
  {
    Comparable::compare(a, self).reverse()
  }

  #[inline]
  unsafe fn compare_binary(a: &[u8], b: &[u8]) -> core::cmp::Ordering {
    a.cmp(b)
  }

  #[inline]
  unsafe fn contains_binary(
    start_bound: Bound<&[u8]>,
    end_bound: Bound<&[u8]>,
    key: &[u8],
  ) -> bool {
    <(Bound<&[u8]>, Bound<&[u8]>) as RangeBounds<[u8]>>::contains(&(start_bound, end_bound), key)
  }
}

impl<'a, K> KeyRef<'a, K> for &'a [u8]
where
  K: ?Sized + Type<Ref<'a> = SliceRef<'a>>,
  &'a [u8]: Comparable<K>,
{
  #[inline]
  fn compare<Q>(&self, a: &Q) -> core::cmp::Ordering
  where
    Q: ?Sized + Ord + Comparable<Self>,
  {
    Comparable::compare(a, self).reverse()
  }

  #[inline]
  unsafe fn compare_binary(a: &[u8], b: &[u8]) -> core::cmp::Ordering {
    a.cmp(b)
  }

  #[inline]
  unsafe fn contains_binary(
    start_bound: Bound<&[u8]>,
    end_bound: Bound<&[u8]>,
    key: &[u8],
  ) -> bool {
    <(Bound<&[u8]>, Bound<&[u8]>) as RangeBounds<[u8]>>::contains(&(start_bound, end_bound), key)
  }
}

impl Type for [u8] {
  type Ref<'a> = SliceRef<'a>;

  type Error = InsufficientBuffer;

  #[inline]
  fn encoded_len(&self) -> usize {
    self.len()
  }

  #[inline]
  fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    buf.put_slice(self)
  }

  #[inline]
  fn as_encoded(&self) -> Option<&[u8]> {
    Some(self)
  }
}

impl KeyRef<'_, [u8]> for [u8] {
  #[inline]
  fn compare<Q>(&self, a: &Q) -> cmp::Ordering
  where
    Q: ?Sized + Ord + Comparable<Self>,
  {
    Comparable::compare(a, self).reverse()
  }

  #[inline]
  unsafe fn compare_binary(a: &[u8], b: &[u8]) -> cmp::Ordering {
    a.cmp(b)
  }

  #[inline]
  unsafe fn contains_binary(
    start_bound: Bound<&[u8]>,
    end_bound: Bound<&[u8]>,
    key: &[u8],
  ) -> bool {
    <(Bound<&[u8]>, Bound<&[u8]>) as RangeBounds<[u8]>>::contains(&(start_bound, end_bound), key)
  }
}

impl Equivalent<[u8]> for SliceRef<'_> {
  #[inline]
  fn equivalent(&self, key: &[u8]) -> bool {
    self.0 == key
  }
}

impl Comparable<[u8]> for SliceRef<'_> {
  #[inline]
  fn compare(&self, key: &[u8]) -> cmp::Ordering {
    self.0.cmp(key)
  }
}

impl<const N: usize> Type for [u8; N] {
  type Ref<'a> = Self;

  type Error = InsufficientBuffer;

  #[inline(always)]
  fn encoded_len(&self) -> usize {
    N
  }

  #[inline]
  fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    buf.put_slice(self.as_ref())
  }

  #[inline]
  fn as_encoded(&self) -> Option<&[u8]> {
    Some(self.as_ref())
  }
}

impl<const N: usize> TypeRef<'_> for [u8; N] {
  #[inline]
  unsafe fn from_slice(src: &'_ [u8]) -> Self {
    let mut this = [0; N];
    this.copy_from_slice(src);
    this
  }
}

impl<const N: usize> KeyRef<'_, [u8; N]> for [u8; N] {
  #[inline]
  fn compare<Q>(&self, a: &Q) -> core::cmp::Ordering
  where
    Q: ?Sized + Ord + Comparable<[u8; N]>,
  {
    Comparable::compare(a, self).reverse()
  }

  #[inline]
  unsafe fn compare_binary(a: &[u8], b: &[u8]) -> core::cmp::Ordering {
    a.cmp(b)
  }

  #[inline]
  unsafe fn contains_binary(
    start_bound: Bound<&[u8]>,
    end_bound: Bound<&[u8]>,
    key: &[u8],
  ) -> bool {
    <(Bound<&[u8]>, Bound<&[u8]>) as RangeBounds<[u8]>>::contains(&(start_bound, end_bound), key)
  }
}

macro_rules! impl_cmp_for_array {
  ($outer:ident($inner:ty) $(@($ret:ty) $trait:ident::$method:ident($($ty:ty),+$(,)?)), +$(,)?) => {
    $(
      $(
        impl<const N: usize> $trait<SliceRef<'_>> for $ty {
          #[inline]
          fn $method(&self, other: &$outer<'_>) -> $ret {
            let this: $inner = self.as_ref();
            $trait::$method(this, other.0)
          }
        }

        impl<const N: usize> $trait<$ty> for SliceRef<'_> {
          #[inline]
          fn $method(&self, other: &$ty) -> $ret {
            let this: $inner = other.as_ref();
            $trait::$method(self.0, this)
          }
        }
      )*
    )*
  };
}

impl_cmp_for_array!(
  SliceRef(&[u8])
  @(bool) PartialEq::eq([u8; N], &[u8; N]),
  @(bool) Equivalent::equivalent([u8; N], &[u8; N]),
  @(Option<Ordering>) PartialOrd::partial_cmp([u8; N], &[u8; N]),
  @(Ordering) Comparable::compare([u8; N], &[u8; N]),
);

impl_cmp!(
  SliceRef(&[u8])
  @(bool) PartialEq::eq([u8]),
  @(Option<Ordering>) PartialOrd::partial_cmp([u8]),
);

impls! {
  #[cfg(feature = "alloc")]
  ::std::borrow::Cow<'_, [u8]>,
  &[u8],
  #[cfg(feature = "alloc")]
  ::std::vec::Vec<u8>,
  #[cfg(feature = "alloc")]
  ::std::boxed::Box<[u8]>,
  #[cfg(feature = "alloc")]
  ::std::sync::Arc<[u8]>,
  #[cfg(feature = "triomphe01")]
  ::triomphe01::Arc<[u8]>,
  #[cfg(feature = "bytes1")]
  ::bytes1::Bytes,
  #[cfg(feature = "smallvec-wrapper01")]
  ::smallvec_wrapper01::OneOrMore<u8>,
  #[cfg(feature = "smallvec-wrapper01")]
  ::smallvec_wrapper01::TinyVec<u8>,
  #[cfg(feature = "smallvec-wrapper01")]
  ::smallvec_wrapper01::TriVec<u8>,
  #[cfg(feature = "smallvec-wrapper01")]
  ::smallvec_wrapper01::SmallVec<u8>,
  #[cfg(feature = "smallvec-wrapper01")]
  ::smallvec_wrapper01::MediumVec<u8>,
  #[cfg(feature = "smallvec-wrapper01")]
  ::smallvec_wrapper01::LargeVec<u8>,
  #[cfg(feature = "smallvec-wrapper01")]
  ::smallvec_wrapper01::XLargeVec<u8>,
  #[cfg(feature = "smallvec-wrapper01")]
  ::smallvec_wrapper01::XXLargeVec<u8>,
  #[cfg(feature = "smallvec-wrapper01")]
  ::smallvec_wrapper01::XXXLargeVec<u8>,
}

#[cfg(any(feature = "smallvec01", feature = "smallvec02"))]
macro_rules! smallvec {
  ($pkg:ident::$ty:ty) => {
    const _: () = {
      use $pkg::SmallVec;

      use super::*;

      impl<const N: usize> Type for $ty {
        type Ref<'a> = SliceRef<'a>;
        type Error = InsufficientBuffer;

        #[inline]
        fn encoded_len(&self) -> usize {
          self.len()
        }

        #[inline]
        fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
          buf.put_slice(self.as_ref())
        }

        #[inline]
        fn as_encoded(&self) -> Option<&[u8]> {
          Some(self.as_ref())
        }
      }

      impl_cmp_for_array! {
        SliceRef(&[u8])
        @(bool) PartialEq::eq($ty, &$ty),
        @(bool) Equivalent::equivalent($ty, &$ty),
        @(Ordering) Comparable::compare($ty, &$ty),
        @(Option<Ordering>) PartialOrd::partial_cmp($ty, &$ty),
      }
    };
  };
}

#[cfg(feature = "smallvec01")]
smallvec!(smallvec01::SmallVec<[u8; N]>);

#[cfg(feature = "smallvec02")]
smallvec!(smallvec02::SmallVec<u8, N>);
