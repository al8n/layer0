use crate::error::InsufficientBuffer;

use super::*;

macro_rules! impl_cmp {
  ($outer:ident($inner:ty) $(@($ret:ty) $trait:ident::$method:ident($($ty:ty),+$(,)?)), +$(,)?) => {
    $(
      $(
        impl $trait<$outer<'_>> for $ty {
          #[inline]
          fn $method(&self, other: &$outer<'_>) -> $ret {
            let this: $inner = self.as_ref();
            $trait::$method(this, other.0)
          }
        }

        impl $trait<$ty> for $outer<'_> {
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

mod bytes;
pub use bytes::*;
mod string;
pub use string::Str;

#[cfg(feature = "std")]
mod net;

impl Type for () {
  type Ref<'a> = ();
  type Error = ();

  #[inline]
  fn encoded_len(&self) -> usize {
    0
  }

  #[inline]
  fn encode(&self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
    Ok(0)
  }

  #[inline]
  fn encode_to_buffer(&self, _buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    Ok(0)
  }

  #[inline]
  fn as_encoded(&self) -> Option<&[u8]> {
    Some(&[])
  }
}

impl<'a> TypeRef<'a> for () {
  unsafe fn from_slice(_buf: &[u8]) -> Self {}

  #[inline]
  fn as_raw(&self) -> Option<&'a [u8]> {
    Some(&[])
  }
}

macro_rules! impl_type {
  ($($ty:ident), +$(,)?) => {
    $(
      impl Type for $ty {
        type Ref<'a> = Self;

        type Error = $crate::error::InsufficientBuffer;

        #[inline]
        fn encoded_len(&self) -> usize {
          core::mem::size_of::<$ty>()
        }

        #[inline]
        fn encode_to_buffer(&self, buf: &mut $crate::buffer::VacantBuffer<'_>) -> Result<usize, Self::Error> {
          buf.put_slice(self.to_le_bytes().as_ref())
        }
      }

      impl TypeRef<'_> for $ty {
        #[inline]
        unsafe fn from_slice(buf: &[u8]) -> Self {
          const SIZE: usize = core::mem::size_of::<$ty>();

          $ty::from_le_bytes(buf[..SIZE].try_into().unwrap())
        }
      }

      #[cfg(test)]
      paste::paste! {
        proptest::proptest! {
          #[test]
          fn [<$ty _encode>](x in [< 0 $ty >]..[< $ty >]::MAX,) {
            let mut buf = [0; core::mem::size_of::<$ty>()];
            let encoded = x.encode(&mut buf).unwrap();
            proptest::prop_assert_eq!(encoded, x.encoded_len());

            let y = unsafe { $ty::from_slice(&buf) };
            proptest::prop_assert_eq!(x, y);
          }

          #[test]
          fn [< $ty _encode_to_buffer>](x in [< 0 $ty >]..[< $ty >]::MAX,) {
            let mut buf = [0u8; core::mem::size_of::<$ty>()];
            let mut buf = $crate::buffer::VacantBuffer::from(buf.as_mut());
            let encoded = x.encode_to_buffer(&mut buf).unwrap();
            proptest::prop_assert_eq!(encoded, x.encoded_len());
            let y = unsafe { $ty::from_slice(buf.as_ref()) };
            proptest::prop_assert_eq!(x, y);
          }
        }
      }
    )*
  };
}

macro_rules! impl_numbers {
  (@key $($ty:ident), +$(,)?) => {
    $(
      impl_type!($ty);

      impl KeyRef<'_, $ty> for $ty {
        #[inline]
        fn compare<Q>(&self, a: &Q) -> core::cmp::Ordering
        where
          Q: ?Sized + Ord + Comparable<$ty> {
          Comparable::compare(a, self).reverse()
        }

        #[inline]
        unsafe fn compare_binary(a: &[u8], b: &[u8]) -> core::cmp::Ordering {
          const SIZE: usize = core::mem::size_of::<$ty>();

          let a = $ty::from_le_bytes(a[..SIZE].try_into().unwrap());
          let b = $ty::from_le_bytes(b[..SIZE].try_into().unwrap());

          a.cmp(&b)
        }
      }
    )*
  };
  ($($ty:ident), +$(,)?) => {
    impl_type!($($ty),+);
  };
}

impl_numbers!(@key i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
impl_numbers!(f32, f64);

impl Type for bool {
  type Ref<'a> = Self;

  type Error = InsufficientBuffer;

  #[inline]
  fn encoded_len(&self) -> usize {
    1
  }

  #[inline]
  fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    buf.put_u8(*self as u8).map(|_| 1)
  }
}

impl TypeRef<'_> for bool {
  #[inline]
  unsafe fn from_slice(buf: &[u8]) -> Self {
    buf[0] != 0
  }
}

impl KeyRef<'_, bool> for bool {
  #[inline]
  fn compare<Q>(&self, a: &Q) -> core::cmp::Ordering
  where
    Q: ?Sized + Ord + Comparable<bool>,
  {
    Comparable::compare(a, self).reverse()
  }

  #[inline]
  unsafe fn compare_binary(a: &[u8], b: &[u8]) -> core::cmp::Ordering {
    let a = bool::from_slice(a);
    let b = bool::from_slice(b);

    a.cmp(&b)
  }
}

impl Type for char {
  type Ref<'a> = Self;

  type Error = InsufficientBuffer;

  #[inline]
  fn encoded_len(&self) -> usize {
    self.len_utf8()
  }

  #[inline]
  fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    let mut char_buf = [0; 4];
    let src = self.encode_utf8(&mut char_buf).as_bytes();
    buf.put_slice(src)
  }
}

impl TypeRef<'_> for char {
  #[inline]
  unsafe fn from_slice(buf: &[u8]) -> Self {
    core::str::from_utf8_unchecked(buf).chars().next().unwrap()
  }
}

impl KeyRef<'_, char> for char {
  #[inline]
  fn compare<Q>(&self, a: &Q) -> core::cmp::Ordering
  where
    Q: ?Sized + Ord + Comparable<char>,
  {
    Comparable::compare(a, self).reverse()
  }

  #[inline]
  unsafe fn compare_binary(a: &[u8], b: &[u8]) -> core::cmp::Ordering {
    let a = char::from_slice(a);
    let b = char::from_slice(b);

    a.cmp(&b)
  }
}
