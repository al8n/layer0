use ::equivalent::*;
use core::borrow::Borrow;

use super::*;

macro_rules! impls {
  ($( $(#[cfg($cfg:meta)])? $ty:ty),+ $(,)?) => {
    $(
      $(#[cfg($cfg)])?
      const _: () = {
        impl Type for $ty {
          type Ref<'a> = SliceRef<'a>;
          type Error = BufferTooSmall;

          #[inline]
          fn encoded_len(&self) -> usize {
            self.len()
          }

          #[inline]
          fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            let buf_len = buf.len();
            let self_len = self.len();
            if buf_len < self_len {
              return Err(BufferTooSmall::new(self_len, buf_len));
            }

            buf.copy_from_slice(self.as_ref());
            Ok(self_len)
          }
        }

        impl Equivalent<SliceRef<'_>> for $ty {
          #[inline]
          fn equivalent(&self, key: &SliceRef<'_>) -> bool {
            let this: &[u8] = self.as_ref();
            this.eq(key.0)
          }
        }

        impl Comparable<SliceRef<'_>> for $ty {
          #[inline]
          fn compare(&self, other: &SliceRef<'_>) -> cmp::Ordering {
            let this: &[u8] = self.as_ref();
            this.cmp(other.0)
          }
        }

        impl Equivalent<$ty> for SliceRef<'_> {
          #[inline]
          fn equivalent(&self, key: &$ty) -> bool {
            let that: &[u8] = key.as_ref();
            self.0.eq(that)
          }
        }

        impl Comparable<$ty> for SliceRef<'_> {
          #[inline]
          fn compare(&self, other: &$ty) -> cmp::Ordering {
            let that: &[u8] = other.as_ref();
            self.0.cmp(that)
          }
        }
      };
    )*
  };
}

impl<'a> TypeRef<'a> for &'a [u8] {
  unsafe fn from_slice(src: &'a [u8]) -> Self {
    src
  }
}

/// A wrapper type for `&'a [u8]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SliceRef<'a>(&'a [u8]);

impl<'a> SliceRef<'a> {
  /// Returns the inner bytes slice.
  #[inline]
  pub const fn as_bytes(&self) -> &[u8] {
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

impl PartialEq<[u8]> for SliceRef<'_> {
  #[inline]
  fn eq(&self, other: &[u8]) -> bool {
    self.0 == other
  }
}

impl PartialEq<&[u8]> for SliceRef<'_> {
  #[inline]
  fn eq(&self, other: &&[u8]) -> bool {
    self.0 == *other
  }
}

impl PartialEq<SliceRef<'_>> for [u8] {
  #[inline]
  fn eq(&self, other: &SliceRef<'_>) -> bool {
    self == other.0
  }
}

impl PartialEq<SliceRef<'_>> for &[u8] {
  #[inline]
  fn eq(&self, other: &SliceRef<'_>) -> bool {
    *self == other.0
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
}

impl Type for [u8] {
  type Ref<'a> = SliceRef<'a>;

  type Error = BufferTooSmall;

  #[inline]
  fn encoded_len(&self) -> usize {
    self.len()
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    let buf_len = buf.len();
    let self_len = self.len();
    if buf_len < self_len {
      return Err(BufferTooSmall::new(self_len, buf_len));
    }

    buf.copy_from_slice(self);
    Ok(self_len)
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
}

impl<const N: usize> Type for [u8; N] {
  type Ref<'a> = Self;

  type Error = BufferTooSmall;

  fn encoded_len(&self) -> usize {
    N
  }

  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    let buf_len = buf.len();

    if buf_len < N {
      return Err(BufferTooSmall::new(N, buf_len));
    }

    buf[..N].copy_from_slice(self.as_ref());
    Ok(N)
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
}

impls! {
  #[cfg(feature = "alloc")]
  ::std::borrow::Cow<'_, [u8]>,
  &'static [u8],
  #[cfg(feature = "alloc")]
  ::std::vec::Vec<u8>,
  #[cfg(feature = "alloc")]
  ::std::boxed::Box<[u8]>,
  #[cfg(feature = "alloc")]
  ::std::sync::Arc<[u8]>,
  #[cfg(feature = "bytes")]
  ::bytes::Bytes,
  #[cfg(feature = "smallvec-wrapper")]
  ::smallvec_wrapper::OneOrMore<u8>,
  #[cfg(feature = "smallvec-wrapper")]
  ::smallvec_wrapper::TinyVec<u8>,
  #[cfg(feature = "smallvec-wrapper")]
  ::smallvec_wrapper::TriVec<u8>,
  #[cfg(feature = "smallvec-wrapper")]
  ::smallvec_wrapper::SmallVec<u8>,
  #[cfg(feature = "smallvec-wrapper")]
  ::smallvec_wrapper::MediumVec<u8>,
  #[cfg(feature = "smallvec-wrapper")]
  ::smallvec_wrapper::LargeVec<u8>,
  #[cfg(feature = "smallvec-wrapper")]
  ::smallvec_wrapper::XLargeVec<u8>,
  #[cfg(feature = "smallvec-wrapper")]
  ::smallvec_wrapper::XXLargeVec<u8>,
  #[cfg(feature = "smallvec-wrapper")]
  ::smallvec_wrapper::XXXLargeVec<u8>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
const _: () = {
  use std::vec::Vec;

  impl PartialEq<Vec<u8>> for SliceRef<'_> {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
      self.0 == other.as_slice()
    }
  }

  impl PartialEq<&Vec<u8>> for SliceRef<'_> {
    #[inline]
    fn eq(&self, other: &&Vec<u8>) -> bool {
      self.0 == other.as_slice()
    }
  }

  impl PartialEq<SliceRef<'_>> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &SliceRef<'_>) -> bool {
      self.as_slice() == other.0
    }
  }

  impl PartialEq<SliceRef<'_>> for &Vec<u8> {
    #[inline]
    fn eq(&self, other: &SliceRef<'_>) -> bool {
      self.as_slice() == other.0
    }
  }
};

#[cfg(feature = "smallvec")]
const _: () = {
  use smallvec::SmallVec;

  use super::*;

  impl<const N: usize> Type for SmallVec<[u8; N]> {
    type Ref<'a> = SliceRef<'a>;
    type Error = BufferTooSmall;

    #[inline]
    fn encoded_len(&self) -> usize {
      self.len()
    }

    #[inline]
    fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
      let buf_len = buf.len();
      let self_len = self.len();
      if buf_len < self_len {
        return Err(BufferTooSmall::new(self_len, buf_len));
      }

      buf.copy_from_slice(self.as_ref());
      Ok(self_len)
    }
  }

  // impl<'a, const N: usize> KeyRef<'a, SmallVec<[u8; N]>> for SliceRef<'a> {
  //   #[inline]
  //   fn compare<Q>(&self, a: &Q) -> cmp::Ordering
  //   where
  //     Q: ?Sized + Ord + Comparable<Self>,
  //   {
  //     Comparable::compare(a, self).reverse()
  //   }

  //   #[inline]
  //   unsafe fn compare_binary(a: &[u8], b: &[u8]) -> cmp::Ordering {
  //     a.cmp(b)
  //   }
  // }

  impl<const N: usize> Equivalent<SliceRef<'_>> for SmallVec<[u8; N]> {
    #[inline]
    fn equivalent(&self, key: &SliceRef<'_>) -> bool {
      let this: &[u8] = self.as_ref();
      this.eq(key.0)
    }
  }

  impl<const N: usize> Comparable<SliceRef<'_>> for SmallVec<[u8; N]> {
    #[inline]
    fn compare(&self, other: &SliceRef<'_>) -> cmp::Ordering {
      let this: &[u8] = self.as_ref();
      this.cmp(other.0)
    }
  }

  impl<const N: usize> Equivalent<SmallVec<[u8; N]>> for SliceRef<'_> {
    #[inline]
    fn equivalent(&self, key: &SmallVec<[u8; N]>) -> bool {
      let that: &[u8] = key.as_ref();
      self.0.eq(that)
    }
  }

  impl<const N: usize> Comparable<SmallVec<[u8; N]>> for SliceRef<'_> {
    #[inline]
    fn compare(&self, other: &SmallVec<[u8; N]>) -> cmp::Ordering {
      let that: &[u8] = other.as_ref();
      self.0.cmp(that)
    }
  }
};
