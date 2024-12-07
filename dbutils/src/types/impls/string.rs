use core::{borrow::Borrow, cmp::Ordering};

use ::equivalent::Equivalent;

use super::*;

macro_rules! impls {
  ($( $(#[cfg($cfg:meta)])? $ty:ty),+ $(,)?) => {
    $(
      $(#[cfg($cfg)])?
      const _: () = {
        impl Type for $ty {
          type Ref<'a> = Str<'a>;
          type Error = InsufficientBuffer;

          #[inline]
          fn encoded_len(&self) -> usize {
            self.len()
          }

          #[inline]
          fn encode_to_buffer(&self, buf: &mut $crate::buffer::VacantBuffer<'_>) -> Result<usize, Self::Error> {
            buf.put_slice(self.as_bytes())
          }

          #[inline]
          fn as_encoded(&self) -> Option<&[u8]> {
            Some(self.as_bytes())
          }
        }

        impl_cmp! {
          Str(&str)
          @(bool) PartialEq::eq($ty, &$ty),
          @(bool) Equivalent::equivalent($ty, &$ty),
          @(Ordering) Comparable::compare($ty, &$ty),
          @(Option<Ordering>) PartialOrd::partial_cmp($ty, &$ty)
        }
      };
    )*
  };
}

impl<'a> TypeRef<'a> for &'a str {
  unsafe fn from_slice(src: &'a [u8]) -> Self {
    core::str::from_utf8(src).unwrap()
  }

  #[inline]
  fn as_raw(&self) -> Option<&'a [u8]> {
    Some(self.as_bytes())
  }
}

/// A wrapper type for `&'a str`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Str<'a>(&'a str);

impl<'a> Str<'a> {
  /// Returns the inner str.
  pub const fn as_str(&self) -> &'a str {
    self.0
  }
}

impl<'a> From<&'a str> for Str<'a> {
  fn from(src: &'a str) -> Self {
    Self(src)
  }
}

impl<'a> From<Str<'a>> for &'a str {
  fn from(src: Str<'a>) -> Self {
    src.0
  }
}

impl<'a> TypeRef<'a> for Str<'a> {
  unsafe fn from_slice(src: &'a [u8]) -> Self {
    Self(core::str::from_utf8(src).unwrap())
  }

  #[inline]
  fn as_raw(&self) -> Option<&'a [u8]> {
    Some(self.0.as_bytes())
  }
}

impl AsRef<str> for Str<'_> {
  fn as_ref(&self) -> &str {
    self.0
  }
}

impl Borrow<str> for Str<'_> {
  fn borrow(&self) -> &str {
    self.0
  }
}

impl core::ops::Deref for Str<'_> {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.0
  }
}

impl PartialEq<str> for Str<'_> {
  fn eq(&self, other: &str) -> bool {
    self.0 == other
  }
}

impl PartialEq<Str<'_>> for str {
  fn eq(&self, other: &Str<'_>) -> bool {
    self == other.0
  }
}

impl PartialOrd<str> for Str<'_> {
  fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
    Some(self.0.cmp(other))
  }
}

impl PartialOrd<Str<'_>> for str {
  fn partial_cmp(&self, other: &Str<'_>) -> Option<cmp::Ordering> {
    Some(self.cmp(other.0))
  }
}

impl Type for str {
  type Ref<'a> = Str<'a>;
  type Error = InsufficientBuffer;

  #[inline]
  fn encoded_len(&self) -> usize {
    self.len()
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    let buf_len = buf.len();
    let self_len = self.len();
    if buf_len < self_len {
      return Err(InsufficientBuffer::with_information(
        self_len as u64,
        buf_len as u64,
      ));
    }

    buf.copy_from_slice(self.as_bytes());
    Ok(self_len)
  }

  #[inline]
  fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    buf.put_slice(self.as_bytes())
  }

  #[inline]
  fn as_encoded(&self) -> Option<&[u8]> {
    Some(self.as_bytes())
  }
}

impl<'a, K> KeyRef<'a, K> for Str<'a>
where
  K: ?Sized + Type<Ref<'a> = Str<'a>>,
  Str<'a>: Comparable<K>,
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

impl<'a, K> KeyRef<'a, K> for &'a str
where
  K: ?Sized + Type<Ref<'a> = &'a str>,
  &'a str: Comparable<K>,
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

impl KeyRef<'_, str> for str {
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

impl Equivalent<str> for Str<'_> {
  #[inline]
  fn equivalent(&self, key: &str) -> bool {
    self.0 == key
  }
}

impl Comparable<str> for Str<'_> {
  #[inline]
  fn compare(&self, key: &str) -> cmp::Ordering {
    self.0.cmp(key)
  }
}

impls! {
  #[cfg(feature = "alloc")]
  ::std::borrow::Cow<'_, str>,
  &str,
  #[cfg(feature = "alloc")]
  ::std::string::String,
  #[cfg(feature = "alloc")]
  ::std::sync::Arc<str>,
  #[cfg(feature = "triomphe01")]
  ::triomphe01::Arc<str>,
  #[cfg(feature = "alloc")]
  ::std::boxed::Box<str>,
  #[cfg(feature = "smol_str03")]
  ::smol_str03::SmolStr,
  #[cfg(feature = "faststr02")]
  ::faststr02::FastStr,
}
