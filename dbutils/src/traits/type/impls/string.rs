use core::borrow::Borrow;

use ::equivalent::Equivalent;

use super::*;

macro_rules! impls {
  ($( $(#[cfg($cfg:meta)])? $ty:ty),+ $(,)?) => {
    $(
      $(#[cfg($cfg)])?
      const _: () = {
        impl Type for $ty {
          type Ref<'a> = Str<'a>;
          type Error = BufferTooSmall;

          fn encoded_len(&self) -> usize {
            self.len()
          }

          fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            let buf_len = buf.len();
            let self_len = self.len();
            if buf_len < self_len {
              return Err(BufferTooSmall::new(self_len, buf_len));
            }

            buf.copy_from_slice(self.as_bytes());
            Ok(self_len)
          }
        }

        impl<'a> KeyRef<'a, $ty> for Str<'a> {
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

        impl Equivalent<Str<'_>> for $ty {
          #[inline]
          fn equivalent(&self, key: &Str<'_>) -> bool {
            let this: &str = self.as_ref();
            this.eq(key.0)
          }
        }

        impl Comparable<Str<'_>> for $ty {
          #[inline]
          fn compare(&self, other: &Str<'_>) -> cmp::Ordering {
            let this: &str = self.as_ref();
            this.cmp(other.0)
          }
        }

        impl Equivalent<$ty> for Str<'_> {
          #[inline]
          fn equivalent(&self, key: &$ty) -> bool {
            let that: &str = key.as_ref();
            self.0.eq(that)
          }
        }

        impl Comparable<$ty> for Str<'_> {
          #[inline]
          fn compare(&self, other: &$ty) -> cmp::Ordering {
            let that: &str = other.as_ref();
            self.0.cmp(that)
          }
        }
      };
    )*
  };
}

impl<'a> TypeRef<'a> for &'a str {
  unsafe fn from_slice(src: &'a [u8]) -> Self {
    core::str::from_utf8(src).unwrap()
  }
}

/// A wrapper type for `&'a str`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Str<'a>(&'a str);

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

impl PartialEq<&str> for Str<'_> {
  fn eq(&self, other: &&str) -> bool {
    self.0 == *other
  }
}

impl PartialEq<Str<'_>> for &str {
  fn eq(&self, other: &Str<'_>) -> bool {
    *self == other.0
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

impl PartialOrd<&str> for Str<'_> {
  fn partial_cmp(&self, other: &&str) -> Option<cmp::Ordering> {
    Some(self.0.cmp(*other))
  }
}

impl PartialOrd<Str<'_>> for &str {
  fn partial_cmp(&self, other: &Str<'_>) -> Option<cmp::Ordering> {
    Some(self.cmp(&other.0))
  }
}

impl Type for str {
  type Ref<'a> = Str<'a>;
  type Error = BufferTooSmall;

  fn encoded_len(&self) -> usize {
    self.len()
  }

  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    let buf_len = buf.len();
    let self_len = self.len();
    if buf_len < self_len {
      return Err(BufferTooSmall::new(self_len, buf_len));
    }

    buf.copy_from_slice(self.as_bytes());
    Ok(self_len)
  }
}

impls! {
  #[cfg(feature = "alloc")]
  ::std::borrow::Cow<'_, str>,
  &'static str,
  #[cfg(feature = "alloc")]
  ::std::string::String,
  #[cfg(feature = "alloc")]
  ::std::sync::Arc<str>,
  #[cfg(feature = "alloc")]
  ::std::boxed::Box<str>,
  #[cfg(feature = "smol_str")]
  ::smol_str::SmolStr,
  #[cfg(feature = "faststr")]
  ::faststr::FastStr,
}

#[cfg(any(feature = "alloc", feature = "std"))]
const _: () = {
  use std::string::String;

  impl PartialEq<String> for Str<'_> {
    fn eq(&self, other: &String) -> bool {
      self.0 == other
    }
  }

  impl PartialEq<Str<'_>> for String {
    fn eq(&self, other: &Str<'_>) -> bool {
      self == other.0
    }
  }

  impl PartialEq<&String> for Str<'_> {
    fn eq(&self, other: &&String) -> bool {
      self.0 == *other
    }
  }

  impl PartialEq<Str<'_>> for &String {
    fn eq(&self, other: &Str<'_>) -> bool {
      *self == other.0
    }
  }
};
