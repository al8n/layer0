use core::{
  array::TryFromSliceError,
  borrow::{Borrow, BorrowMut},
  marker::PhantomData,
  mem,
  ptr::{self, NonNull},
  slice,
};

use crate::{
  equivalent::{Comparable, Equivalent},
  error::InsufficientBuffer,
  types::{MaybeStructured, Type},
};

use super::leb128::*;

/// Writing self to the [`VacantBuffer`] in bytes format.
pub trait BufWriter {
  /// The error type.
  type Error;

  /// The length of the encoded bytes.
  fn encoded_len(&self) -> usize;

  /// Encode self to bytes and write to the [`VacantBuffer`].
  ///
  /// Returns the number of bytes written if successful.
  fn write(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error>;
}

impl<A: AsRef<[u8]>> BufWriter for A {
  type Error = InsufficientBuffer;

  #[inline]
  fn encoded_len(&self) -> usize {
    self.as_ref().len()
  }

  #[inline]
  fn write(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    let src = self.as_ref();
    buf.put_slice(src).map(|_| src.len())
  }
}

impl<T: ?Sized + Type> BufWriter for MaybeStructured<'_, T>
where
  T: Type,
{
  type Error = T::Error;

  #[inline]
  fn encoded_len(&self) -> usize {
    MaybeStructured::encoded_len(self)
  }

  #[inline]
  fn write(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    MaybeStructured::encode_to_buffer(self, buf)
  }
}

/// Like [`BufWriter`] but only write once.
pub trait BufWriterOnce {
  /// The error type.
  type Error;

  /// The length of the encoded bytes.
  fn encoded_len(&self) -> usize;

  /// Encode self to bytes and write to the [`VacantBuffer`].
  ///
  /// Returns the number of bytes written if successful.
  fn write_once(self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error>;
}

impl<A: Borrow<[u8]>> BufWriterOnce for A {
  type Error = InsufficientBuffer;

  #[inline]
  fn encoded_len(&self) -> usize {
    self.borrow().len()
  }

  #[inline]
  fn write_once(self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    let src = self.borrow();
    buf.put_slice(src).map(|_| src.len())
  }
}

impl<T: ?Sized + Type> BufWriterOnce for MaybeStructured<'_, T>
where
  T: Type,
{
  type Error = T::Error;

  #[inline]
  fn encoded_len(&self) -> usize {
    MaybeStructured::encoded_len(self)
  }

  #[inline]
  fn write_once(self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    MaybeStructured::encode_to_buffer(&self, buf)
  }
}

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
        pub fn [< put_ $ty _varint >](&mut self, value: $ty) -> Result<usize, $crate::error::InsufficientBuffer> {
          let len = [< encoded_ $ty _varint_len >](value);
          let remaining = self.cap - self.len;
          if len > remaining {
            return Err($crate::error::InsufficientBuffer::with_information(len as u64, remaining as u64));
          }

          // SAFETY: the value's ptr is aligned and the cap is the correct.
          unsafe {
            let slice = slice::from_raw_parts_mut(self.value.as_ptr().add(self.len), len);
            const_varint::Varint::encode(&value, slice).map_err(Into::into).inspect(|_| {
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
            const_varint::Varint::encode(&value, slice).inspect(|_| {
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
        pub fn [< put_ $ty _le>](&mut self, value: $ty) -> Result<(), $crate::error::InsufficientBuffer> {
          self.put_slice(&value.to_le_bytes()).map(|_| ())
        }

        #[doc = "Puts a `" $ty "` to the buffer in little-endian format without bounds checking."]
        ///
        /// # Panics
        #[doc = "- If the buffer does not have enough space to hold the `" $ty "`."]
        pub fn [< put_ $ty _le_unchecked>](&mut self, value: $ty) {
          self.put_slice_unchecked(&value.to_le_bytes());
        }

        #[doc = "Puts a `" $ty "` to the buffer in big-endian format."]
        pub fn [< put_ $ty _be>](&mut self, value: $ty) -> Result<(), $crate::error::InsufficientBuffer> {
          self.put_slice(&value.to_be_bytes()).map(|_| ())
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
impl Drop for VacantBuffer<'_> {
  fn drop(&mut self) {
    let remaining = self.cap - self.len;
    if remaining > 0 {
      tracing::warn!(
        "vacant buffer is not fully filled with bytes (remaining {})",
        remaining,
      );
    }
  }
}

impl<'a> From<&'a mut [u8]> for VacantBuffer<'a> {
  #[inline]
  fn from(buf: &'a mut [u8]) -> Self {
    let len = buf.len();
    let ptr = buf.as_mut_ptr();
    unsafe { Self::new(len, NonNull::new_unchecked(ptr)) }
  }
}

impl<'a> VacantBuffer<'a> {
  /// Returns the slice of the vacant value. The lifetime is bound to the buffer.
  #[inline]
  pub fn as_slice(&self) -> &'a [u8] {
    if self.cap == 0 {
      return &[];
    }

    unsafe { slice::from_raw_parts(self.value.as_ptr(), self.len) }
  }
}

impl VacantBuffer<'_> {
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

  /// Splits the buffer into two at the given index.
  ///
  /// Afterwards `self` has capacity `cap - at`, and the returned
  /// `VacantBuffer` has capacity `at`.
  ///
  /// This is an `O(1)` operation.
  ///
  /// ## Panics
  ///
  /// Panics if `at > cap`.
  #[inline]
  pub fn split_to(&mut self, at: usize) -> Self {
    if at == 0 {
      return Self::dangling();
    }

    if at == self.cap {
      return mem::replace(self, Self::dangling());
    }

    assert!(
      at <= self.cap,
      "split_to out of bounds: {:?} <= {:?}",
      at,
      self.len,
    );

    let new = unsafe { VacantBuffer::new(self.cap - at, self.value.add(at)) };
    self.cap = at;

    match self.len.checked_sub(at) {
      Some(len) => {
        self.len = len;
      }
      None => {
        self.len = 0;
      }
    }

    mem::replace(self, new)
  }

  /// Splits the bytes into two at the given index.
  ///
  /// Afterwards `self` has the capacity `at`, and the returned `VacantBuffer`
  /// has the capacity `cap - at`.
  ///
  /// This is an `O(1)` operation.
  ///
  /// ## Panics
  ///
  /// Panics if `at > cap`.
  pub fn split_off(&mut self, at: usize) -> Self {
    if at == 0 {
      return mem::replace(self, Self::dangling());
    }

    if at == self.cap {
      return Self::dangling();
    }

    assert!(
      at <= self.cap,
      "split_off out of bounds: {:?} <= {:?}",
      at,
      self.len,
    );

    let mut new = unsafe { VacantBuffer::new(self.cap - at, self.value.add(at)) };
    self.cap = at;
    match self.len.checked_sub(at) {
      Some(len) => {
        new.len = len;
      }
      None => {
        new.len = 0;
      }
    }

    new
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
  ///
  /// Returns the number of bytes written if successful.
  pub fn put_slice(&mut self, bytes: &[u8]) -> Result<usize, InsufficientBuffer> {
    let len = bytes.len();
    let remaining = self.cap - self.len;
    if len > remaining {
      return Err(InsufficientBuffer::with_information(
        remaining as u64,
        len as u64,
      ));
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
    Ok(len)
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
  pub fn put_u8(&mut self, value: u8) -> Result<(), InsufficientBuffer> {
    self.put_slice(&[value]).map(|_| ())
  }

  /// Put a byte to the vacant value without bounds checking.
  ///
  /// # Panics
  /// - If the buffer does not have enough space to hold the byte.
  pub fn put_u8_unchecked(&mut self, value: u8) {
    self.put_slice_unchecked(&[value]);
  }

  /// Puts a `i8` to the buffer.
  pub fn put_i8(&mut self, value: i8) -> Result<(), InsufficientBuffer> {
    self.put_slice(&[value as u8]).map(|_| ())
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

impl core::ops::Deref for VacantBuffer<'_> {
  type Target = [u8];

  fn deref(&self) -> &Self::Target {
    if self.cap == 0 {
      return &[];
    }

    unsafe { slice::from_raw_parts(self.value.as_ptr(), self.len) }
  }
}

impl core::ops::DerefMut for VacantBuffer<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    if self.cap == 0 {
      return &mut [];
    }

    unsafe { slice::from_raw_parts_mut(self.value.as_ptr(), self.len) }
  }
}

impl AsRef<[u8]> for VacantBuffer<'_> {
  fn as_ref(&self) -> &[u8] {
    self
  }
}

impl AsMut<[u8]> for VacantBuffer<'_> {
  fn as_mut(&mut self) -> &mut [u8] {
    self
  }
}

impl Borrow<[u8]> for VacantBuffer<'_> {
  fn borrow(&self) -> &[u8] {
    self
  }
}

impl BorrowMut<[u8]> for VacantBuffer<'_> {
  fn borrow_mut(&mut self) -> &mut [u8] {
    self
  }
}

impl Equivalent<VacantBuffer<'_>> for [u8] {
  fn equivalent(&self, key: &VacantBuffer<'_>) -> bool {
    self.eq(key)
  }
}

impl Comparable<VacantBuffer<'_>> for [u8] {
  fn compare(&self, other: &VacantBuffer<'_>) -> core::cmp::Ordering {
    self.cmp(other)
  }
}

impl<Q> PartialEq<Q> for VacantBuffer<'_>
where
  [u8]: Borrow<Q>,
  Q: ?Sized + Eq,
{
  fn eq(&self, other: &Q) -> bool {
    self.as_ref().borrow().eq(other)
  }
}

impl<Q> PartialOrd<Q> for VacantBuffer<'_>
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
