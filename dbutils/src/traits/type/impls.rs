use super::*;

mod bytes;
pub use bytes::*;
mod string;
pub use string::Str;

#[cfg(feature = "std")]
mod net;

impl Type for () {
  type Ref<'a> = ();
  type Error = ();

  fn encoded_len(&self) -> usize {
    0
  }

  fn encode(&self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
    Ok(0)
  }
}

impl TypeRef<'_> for () {
  unsafe fn from_slice(_buf: &[u8]) -> Self {}
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

macro_rules! impl_numbers {
  ($($ty:ident), +$(,)?) => {
    $(
      impl Type for $ty {
        type Ref<'a> = Self;

        type Error = $crate::traits::BufferTooSmall;

        #[inline]
        fn encoded_len(&self) -> usize {
          core::mem::size_of::<$ty>()
        }

        #[inline]
        fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
          const SIZE: usize = core::mem::size_of::<$ty>();

          let buf_len = buf.len();
          if buf_len < SIZE {
            return Err($crate::traits::BufferTooSmall::new(SIZE, buf_len));
          }

          buf[..SIZE].copy_from_slice(self.to_le_bytes().as_ref());
          Ok(SIZE)
        }
      }

      impl TypeRef<'_> for $ty {
        #[inline]
        unsafe fn from_slice(buf: &[u8]) -> Self {
          const SIZE: usize = core::mem::size_of::<$ty>();

          $ty::from_le_bytes(buf[..SIZE].try_into().unwrap())
        }
      }

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
}

impl_numbers!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

impl Type for f32 {
  type Ref<'a> = Self;

  type Error = BufferTooSmall;

  #[inline]
  fn encoded_len(&self) -> usize {
    core::mem::size_of::<f32>()
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    const SIZE: usize = core::mem::size_of::<f32>();

    let buf_len = buf.len();
    if buf_len < SIZE {
      return Err(BufferTooSmall::new(SIZE, buf_len));
    }

    buf[..SIZE].copy_from_slice(self.to_le_bytes().as_ref());
    Ok(SIZE)
  }
}

impl TypeRef<'_> for f32 {
  #[inline]
  unsafe fn from_slice(buf: &[u8]) -> Self {
    const SIZE: usize = core::mem::size_of::<f32>();

    f32::from_le_bytes(buf[..SIZE].try_into().unwrap())
  }
}

impl Type for f64 {
  type Ref<'a> = Self;

  type Error = BufferTooSmall;

  #[inline]
  fn encoded_len(&self) -> usize {
    core::mem::size_of::<f64>()
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    const SIZE: usize = core::mem::size_of::<f64>();

    let buf_len = buf.len();
    if buf_len < SIZE {
      return Err(BufferTooSmall::new(SIZE, buf_len));
    }

    buf[..SIZE].copy_from_slice(self.to_le_bytes().as_ref());
    Ok(SIZE)
  }
}

impl TypeRef<'_> for f64 {
  #[inline]
  unsafe fn from_slice(buf: &[u8]) -> Self {
    const SIZE: usize = core::mem::size_of::<f64>();

    f64::from_le_bytes(buf[..SIZE].try_into().unwrap())
  }
}

impl Type for bool {
  type Ref<'a> = Self;

  type Error = BufferTooSmall;

  #[inline]
  fn encoded_len(&self) -> usize {
    1
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    if buf.is_empty() {
      return Err(BufferTooSmall::new(1, 0));
    }

    buf[0] = *self as u8;
    Ok(1)
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

  type Error = BufferTooSmall;

  #[inline]
  fn encoded_len(&self) -> usize {
    self.len_utf8()
  }

  #[inline]
  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    let len = self.len_utf8();
    if buf.len() < len {
      return Err(BufferTooSmall::new(len, buf.len()));
    }
    self.encode_utf8(buf);
    Ok(len)
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