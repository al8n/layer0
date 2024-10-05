use core::{
  array::TryFromSliceError,
  borrow::{Borrow, BorrowMut},
  marker::PhantomData,
  ptr::{self, NonNull},
  slice,
};

use equivalent::{Comparable, Equivalent};

use super::leb128::*;

macro_rules! impl_get_varint {
  ($($ty:ident), +$(,)?) => {
    $(
      paste::paste! {
        /// Decodes a value from LEB128 variable length format.
        ///
        /// # Arguments
        ///
        /// * `buf` - A byte slice containing the LEB128 encoded value.
        ///
        /// # Returns
        ///
        #[doc = "* Returns the bytes readed and the decoded value as `" $ty "` if successful."]
        ///
        /// * Returns [`DecodeVarintError`] if the buffer did not contain a valid LEB128 encoding
        ///   or the decode buffer did not contain enough bytes to decode a value.
        #[inline]
        pub fn [< get_ $ty _varint >](&self) -> Result<(usize, $ty), DecodeVarintError> {
          [< decode_ $ty _varint >](self.as_ref())
        }

        /// Decodes a value from LEB128 variable length format.
        ///
        /// # Arguments
        ///
        /// * `buf` - A byte slice containing the LEB128 encoded value.
        ///
        /// # Returns
        ///
        #[doc = "* Returns the bytes readed and the decoded value as `" $ty "` if successful, otherwise panic."]
        ///
        /// # Panics
        /// - If the buffer did not contain a valid LEB128 encoding or the decode buffer did not contain enough bytes to decode a value.
        #[inline]
        pub fn [< get_ $ty _varint_unchecked >](&self) -> (usize, $ty) {
          [< decode_ $ty _varint >](self.as_ref()).unwrap()
        }
      }
    )*
  };
}

macro_rules! impl_put_varint {
  ($($ty:ident), +$(,)?) => {
    $(
      paste::paste! {
        #[doc = "Encodes an `" $ty "`value into LEB128 variable length format, and writes it to the buffer."]
        pub fn [< put_ $ty _varint >](&mut self, value: $ty) -> Result<usize, EncodeVarintError> {
          let len = [< encoded_ $ty _varint_len >](value);
          let remaining = self.cap - self.len;
          if len > remaining {
            return Err(EncodeVarintError::BufferTooSmall);
          }

          // SAFETY: the value's ptr is aligned and the cap is the correct.
          unsafe {
            let slice = slice::from_raw_parts_mut(self.value.as_ptr().add(self.len), len);
            [< encode_ $ty _varint >](value, slice).inspect(|_| {
              self.len += len;
            })
          }
        }

        #[doc = "Encodes an `" $ty "`value into LEB128 variable length format, and writes it to the buffer, without bounds checking."]
        ///
        /// # Panics
        #[doc = "- If the buffer does not have enough space to hold the encoded `" $ty "` in LEB128 format."]
        pub fn [< put_ $ty _varint_unchecked >](&mut self, value: $ty) -> usize {
          let len = [< encoded_ $ty _varint_len >](value);
          let remaining = self.cap - self.len;
          if len > remaining {
            panic!(
              "buffer does not have enough space (remaining {}, want {})",
              remaining, len
            );
          }

          // SAFETY: the value's ptr is aligned and the cap is the correct.
          unsafe {
            let slice = slice::from_raw_parts_mut(self.value.as_ptr().add(self.len), len);
            [< encode_ $ty _varint >] (value, slice).inspect(|_| {
              self.len += len;
            }).unwrap()
          }
        }
      }
    )*
  };
}

macro_rules! impl_get {
  ($($ty:ident), +$(,)?) => {
    $(
      paste::paste! {
        #[doc = "Decodes a `" $ty "` from the buffer in little-endian format."]
        #[inline]
        pub fn [< get_ $ty _le >](&self) -> Result<$ty, TryFromSliceError> {
          self.as_ref().try_into().map($ty::from_le_bytes)
        }

        #[doc = "Decodes a `" $ty "` from the buffer in little-endian format without checking."]
        ///
        /// # Panics
        #[doc = "- If the buffer did not contain enough bytes to decode a `" $ty "`."]
        #[inline]
        pub fn [< get_ $ty _le_unchecked >](&self) -> $ty {
          self.as_ref().try_into().map($ty::from_le_bytes).unwrap()
        }

        #[doc = "Decodes a `" $ty "` from the buffer in big-endian format."]
        #[inline]
        pub fn [< get_ $ty _be >](&self) -> Result<$ty, TryFromSliceError> {
          self.as_ref().try_into().map($ty::from_be_bytes)
        }

        #[doc = "Decodes a `" $ty "` from the buffer in big-endian format without checking."]
        ///
        /// # Panics
        #[doc = "- If the buffer did not contain enough bytes to decode a `" $ty "`."]
        #[inline]
        pub fn [< get_ $ty _be_unchecked >](&self) -> $ty {
          self.as_ref().try_into().map($ty::from_be_bytes).unwrap()
        }
      }
    )*
  };
}

macro_rules! impl_put {
  ($($ty:ident), +$(,)?) => {
    $(
      paste::paste! {
        #[doc = "Puts a `" $ty "` to the buffer in little-endian format."]
        pub fn [< put_ $ty _le>](&mut self, value: $ty) -> Result<(), NotEnoughSpace> {
          self.put_slice(&value.to_le_bytes())
        }

        #[doc = "Puts a `" $ty "` to the buffer in little-endian format without bounds checking."]
        ///
        /// # Panics
        #[doc = "- If the buffer does not have enough space to hold the `" $ty "`."]
        pub fn [< put_ $ty _le_unchecked>](&mut self, value: $ty) {
          self.put_slice_unchecked(&value.to_le_bytes());
        }

        #[doc = "Puts a `" $ty "` to the buffer in big-endian format."]
        pub fn [< put_ $ty _be>](&mut self, value: $ty) -> Result<(), NotEnoughSpace> {
          self.put_slice(&value.to_be_bytes())
        }

        #[doc = "Puts a `" $ty "` to the buffer in big-endian format without bounds checking."]
        ///
        /// # Panics
        #[doc = "- If the buffer does not have enough space to hold the `" $ty "`."]
        pub fn [< put_ $ty _be_unchecked>](&mut self, value: $ty) {
          self.put_slice_unchecked(&value.to_be_bytes());
        }
      }
    )*
  };
}

/// Returns when the bytes are too large to be written to the vacant buffer.
#[derive(Debug, Default, Clone, Copy)]
pub struct NotEnoughSpace {
  remaining: usize,
  want: usize,
}

impl core::fmt::Display for NotEnoughSpace {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "vacant buffer does not have enough space (remaining {}, want {})",
      self.remaining, self.want
    )
  }
}

#[cfg(feature = "std")]
impl std::error::Error for NotEnoughSpace {}

/// A vacant buffer in the WAL.
#[must_use = "vacant buffer must be filled with bytes."]
#[derive(Debug)]
pub struct VacantBuffer<'a> {
  value: NonNull<u8>,
  len: usize,
  cap: usize,
  _m: PhantomData<&'a ()>,
}

#[cfg(feature = "tracing")]
impl<'a> Drop for VacantBuffer<'a> {
  fn drop(&mut self) {
    let remaining = self.remaining();
    if remaining > 0 {
      tracing::warn!(
        "vacant buffer is not fully filled with bytes (remaining {})",
        remaining,
      );
    }
  }
}

impl<'a> VacantBuffer<'a> {
  /// Fill the remaining space with the given byte.
  #[inline]
  pub fn fill(&mut self, byte: u8) {
    if self.cap == 0 {
      return;
    }

    // SAFETY: the value's ptr is aligned and the cap is the correct.
    unsafe {
      ptr::write_bytes(self.value.as_ptr(), byte, self.cap);
    }
    self.len = self.cap;
  }

  /// Set the length of the vacant buffer.
  ///
  /// If the length is greater than the current length, the gap will be filled with zeros.
  ///
  /// ## Panics
  /// - If the length is greater than the capacity.
  pub fn set_len(&mut self, len: usize) {
    if len > self.cap {
      panic!(
        "buffer does not have enough space (remaining {}, want {})",
        self.cap - self.len,
        len
      );
    }

    // If the length is greater than the current length, the gap will be filled with zeros.
    if len > self.len {
      // SAFETY: the value's ptr is aligned and the cap is the correct.
      unsafe {
        ptr::write_bytes(self.value.as_ptr().add(self.len), 0, len - self.len);
      }
    }

    // If the length is less than the current length, the buffer will be truncated.
    if len < self.len {
      // SAFETY: the value's ptr is aligned and the cap is the correct.
      unsafe {
        ptr::write_bytes(self.value.as_ptr().add(len), 0, self.len - len);
      }

      self.len = len;
    }

    self.len = len;
  }

  /// Put bytes to the vacant value.
  pub fn put_slice(&mut self, bytes: &[u8]) -> Result<(), NotEnoughSpace> {
    let len = bytes.len();
    let remaining = self.cap - self.len;
    if len > remaining {
      return Err(NotEnoughSpace {
        remaining,
        want: len,
      });
    }

    // SAFETY: the value's ptr is aligned and the cap is the correct.
    unsafe {
      self
        .value
        .as_ptr()
        .add(self.len)
        .copy_from(bytes.as_ptr(), len);
    }

    self.len += len;
    Ok(())
  }

  /// Write bytes to the vacant value without bounds checking.
  ///
  /// # Panics
  /// - If a slice is larger than the remaining space.
  pub fn put_slice_unchecked(&mut self, bytes: &[u8]) {
    let len = bytes.len();
    let remaining = self.cap - self.len;
    if len > remaining {
      panic!(
        "buffer does not have enough space (remaining {}, want {})",
        remaining, len
      );
    }

    // SAFETY: the value's ptr is aligned and the cap is the correct.
    unsafe {
      self
        .value
        .as_ptr()
        .add(self.len)
        .copy_from(bytes.as_ptr(), len);
    }
    self.len += len;
  }

  impl_get_varint!(u16, u32, u64, u128, i16, i32, i64, i128);
  impl_get!(u16, u32, u64, u128, i16, i32, i64, i128, f32, f64);
  impl_put_varint!(u16, u32, u64, u128, i16, i32, i64, i128);
  impl_put!(u16, u32, u64, u128, i16, i32, i64, i128, f32, f64);

  /// Put a byte to the vacant value.
  pub fn put_u8(&mut self, value: u8) -> Result<(), NotEnoughSpace> {
    self.put_slice(&[value])
  }

  /// Put a byte to the vacant value without bounds checking.
  ///
  /// # Panics
  /// - If the buffer does not have enough space to hold the byte.
  pub fn put_u8_unchecked(&mut self, value: u8) {
    self.put_slice_unchecked(&[value]);
  }

  /// Puts a `i8` to the buffer.
  pub fn put_i8(&mut self, value: i8) -> Result<(), NotEnoughSpace> {
    self.put_slice(&[value as u8])
  }

  /// Puts a `i8` to the buffer without bounds checking.
  ///
  /// # Panics
  /// - If the buffer does not have enough space to hold the `i8`.
  pub fn put_i8_unchecked(&mut self, value: i8) {
    self.put_slice_unchecked(&[value as u8]);
  }

  /// Returns the capacity of the vacant value.
  #[inline]
  pub const fn capacity(&self) -> usize {
    self.cap
  }

  /// Returns the length of the vacant value.
  #[inline]
  pub const fn len(&self) -> usize {
    self.len
  }

  /// Returns `true` if the vacant value is empty.
  #[inline]
  pub const fn is_empty(&self) -> bool {
    self.len == 0
  }

  /// Returns the remaining space of the vacant value.
  #[inline]
  pub const fn remaining(&self) -> usize {
    self.cap - self.len
  }

  /// Construct a new vacant buffer.
  ///
  /// # Safety
  /// - The ptr must be a valid pointer and its capacity must be the less or equal to the `cap`.
  #[inline]
  pub const unsafe fn new(cap: usize, ptr: NonNull<u8>) -> Self {
    Self {
      value: ptr,
      len: 0,
      cap,
      _m: PhantomData,
    }
  }

  /// Construct a dangling vacant buffer.
  #[inline]
  pub const fn dangling() -> Self {
    Self {
      value: NonNull::dangling(),
      len: 0,
      cap: 0,
      _m: PhantomData,
    }
  }
}

impl<'a> core::ops::Deref for VacantBuffer<'a> {
  type Target = [u8];

  fn deref(&self) -> &Self::Target {
    if self.cap == 0 {
      return &[];
    }

    unsafe { slice::from_raw_parts(self.value.as_ptr(), self.len) }
  }
}

impl<'a> core::ops::DerefMut for VacantBuffer<'a> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    if self.cap == 0 {
      return &mut [];
    }

    unsafe { slice::from_raw_parts_mut(self.value.as_ptr(), self.len) }
  }
}

impl<'a> AsRef<[u8]> for VacantBuffer<'a> {
  fn as_ref(&self) -> &[u8] {
    self
  }
}

impl<'a> AsMut<[u8]> for VacantBuffer<'a> {
  fn as_mut(&mut self) -> &mut [u8] {
    self
  }
}

impl<'a> Borrow<[u8]> for VacantBuffer<'a> {
  fn borrow(&self) -> &[u8] {
    self
  }
}

impl<'a> BorrowMut<[u8]> for VacantBuffer<'a> {
  fn borrow_mut(&mut self) -> &mut [u8] {
    self
  }
}

impl<'a, Q> Equivalent<Q> for VacantBuffer<'a>
where
  [u8]: Borrow<Q>,
  Q: ?Sized + Eq,
{
  fn equivalent(&self, key: &Q) -> bool {
    self.as_ref().borrow().eq(key)
  }
}

impl<'a, Q> Comparable<Q> for VacantBuffer<'a>
where
  [u8]: Borrow<Q>,
  Q: ?Sized + Ord,
{
  fn compare(&self, other: &Q) -> core::cmp::Ordering {
    self.as_ref().borrow().compare(other)
  }
}

impl<'a, Q> PartialEq<Q> for VacantBuffer<'a>
where
  [u8]: Borrow<Q>,
  Q: ?Sized + Eq,
{
  fn eq(&self, other: &Q) -> bool {
    self.as_ref().borrow().eq(other)
  }
}

impl<'a, Q> PartialOrd<Q> for VacantBuffer<'a>
where
  [u8]: Borrow<Q>,
  Q: ?Sized + Ord,
{
  fn partial_cmp(&self, other: &Q) -> Option<core::cmp::Ordering> {
    #[allow(clippy::needless_borrow)]
    Some(self.as_ref().borrow().cmp(&other))
  }
}

macro_rules! impl_ord {
  ($(
    $(const $N: ident)? impl <$ty1:ty> <=> $ty2:ty
  ),+$(,)?) => {
    $(
      impl<'a $(, const $N: usize)? > PartialEq<$ty1> for $ty2 {
        fn eq(&self, other: &$ty1) -> bool {
          self.as_ref().eq(other.as_ref())
        }
      }

      impl<'a $(, const $N: usize)? > PartialEq<$ty2> for $ty1 {
        fn eq(&self, other: &$ty2) -> bool {
          self.as_ref().eq(other)
        }
      }

      impl<'a $(, const $N: usize)? > PartialOrd<$ty1> for $ty2 {
        fn partial_cmp(&self, other: &$ty1) -> Option<core::cmp::Ordering> {
          self.as_ref().partial_cmp(other.as_ref())
        }
      }

      impl<'a $(, const $N: usize)? > PartialOrd<$ty2> for $ty1 {
        fn partial_cmp(&self, other: &$ty2) -> Option<core::cmp::Ordering> {
          self.as_ref().partial_cmp(other.as_ref())
        }
      }
    )*
  };
  ($(
    $(const $N: ident)? impl <$ty1:ty> => $ty2:ty
  ),+$(,)?) => {
    $(
      impl<'a $(, const $N: usize)? > PartialEq<$ty1> for $ty2 {
        fn eq(&self, other: &$ty1) -> bool {
          self.as_ref().eq(other.as_ref())
        }
      }

      impl<'a $(, const $N: usize)? > PartialOrd<$ty1> for $ty2 {
        fn partial_cmp(&self, other: &$ty1) -> Option<core::cmp::Ordering> {
          self.as_ref().partial_cmp(other.as_ref())
        }
      }
    )*
  };
}

impl_ord!(
  impl <VacantBuffer<'a>> => [u8],
  const N impl <VacantBuffer<'a>> => [u8; N],
);

impl_ord!(
  impl <&VacantBuffer<'a>> <=> [u8],
  impl <&mut VacantBuffer<'a>> <=> [u8],
  const N impl <&VacantBuffer<'a>> <=> [u8; N],
  const N impl <&mut VacantBuffer<'a>> <=> [u8; N],
);
