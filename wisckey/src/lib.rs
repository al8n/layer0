//! A key-value seperated WAL log based on https://www.usenix.org/system/files/conference/fast16/fast16-papers-lu.pdf
#![allow(clippy::type_complexity)]
// #![deny(warnings, missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![cfg_attr(not(all(feature = "std", test)), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc as std;

use core::mem;
#[cfg(feature = "std")]
pub use std::error::Error;

#[cfg(not(feature = "std"))]
pub trait Error: core::fmt::Debug + core::fmt::Display {}

#[cfg(not(feature = "std"))]
impl<T: core::fmt::Debug + core::fmt::Display> Error for T {}

const MAX_ENCODED_VALUE_POINTER_SIZE: usize = mem::size_of::<u32>() + 10 + mem::size_of::<u64>();

/// From usize
pub trait FromUsize: Sized {
  /// Create a new instance from the given usize.
  fn from_usize(n: usize) -> Self;
}

/// Convert usize to Self
pub trait IntoUsize {
  /// Convert self to usize.
  fn into_usize(self) -> usize;
}

macro_rules! impl_from_into_usize {
  ($($ident:ident), +$(,)?) => {
    $(
      impl FromUsize for $ident {
        fn from_usize(n: usize) -> Self {
          n as Self
        }
      }

      impl IntoUsize for $ident {
        fn into_usize(self) -> usize {
          self as usize
        }
      }
    )+
  };
}

impl_from_into_usize!(u8, u16, u32, u64);

/// The size of the size of value in the log.
pub trait ValueSize: sealed::Sealed {}

mod sealed {
  use super::{FromUsize, IntoUsize};

  pub trait Sealed: FromUsize + IntoUsize + Sized {
    /// The error type for this value size.
    type Error;

    /// The maximum size can be represented by this value size.
    const MAX_SIZE: usize;

    /// The maximum encoded size in bytes.
    ///
    /// e.g. u8 is 1 byte, u16 is 2 bytes.
    const MAX_ENCODED_SIZE: usize;

    /// Returns the encoded size of this value size.
    fn encoded_size(&self) -> usize;

    /// Encodes the value size into the buffer.
    fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error>;

    /// Decodes the value size from the buffer, and returns the number of bytes consumed and self.
    fn decode(buf: &[u8]) -> Result<(usize, Self), Self::Error>;
  }

  impl<T: Sealed> super::ValueSize for T {}
}

/// The encoded size of the value is fixed.
///
/// e.g. u16 will always be encoded into 2 bytes.
pub trait FixedValueSize<const N: usize>: ValueSize {
  /// Encodes the value size to an array of bytes.
  fn encode_to_array(&self) -> [u8; N];

  /// Decodes the value size from an array of bytes.
  fn decode_from_array(src: [u8; N]) -> Self;
}

macro_rules! impl_fixed_value_size {
  ($($ident:ident::$from_buf:expr), +$(,)?) => {
    $(
      paste::paste! {
        /// The [`ValueSize`] error for u16.
        #[doc = concat!("The [`ValueSize`] error for ", stringify!($ident), ".")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum [< $ident:camel ValueSizeError >] {
          /// The encode buffer is too small.
          EncodeBufferTooSmall,
          /// Not enough bytes to decode the value size.
          NotEnoughBytes,
        }

        impl core::fmt::Display for [< $ident:camel ValueSizeError >] {
          fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
              Self::EncodeBufferTooSmall => write!(f, "encode buffer too small"),
              Self::NotEnoughBytes => write!(f, "not enough bytes"),
            }
          }
        }

        #[cfg(feature = "std")]
        impl std::error::Error for [< $ident:camel ValueSizeError >] {}

        impl sealed::Sealed for $ident {
          type Error = [< $ident:camel ValueSizeError >];

          const MAX_SIZE: usize = $ident::MAX as usize;
          const MAX_ENCODED_SIZE: usize = core::mem::size_of::<$ident>();

          fn encoded_size(&self) -> usize {
            core::mem::size_of::<$ident>()
          }

          fn encode(&self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
            if buf.len() < core::mem::size_of::<$ident>() {
              return Err(Self::Error::EncodeBufferTooSmall);
            }

            buf[..core::mem::size_of::<$ident>()].copy_from_slice(&self.to_le_bytes());
            core::result::Result::Ok(core::mem::size_of::<$ident>())
          }

          fn decode(buf: &[u8]) -> core::result::Result<(usize, Self), Self::Error> {
            if buf.len() < core::mem::size_of::<$ident>() {
              return Err(Self::Error::NotEnoughBytes);
            }

            core::result::Result::Ok((core::mem::size_of::<$ident>(), $from_buf(buf)))
          }
        }
      }

      impl FixedValueSize< { core::mem::size_of::<$ident>() }> for $ident {
        fn encode_to_array(&self) -> [u8; core::mem::size_of::<$ident>()] {
          self.to_le_bytes()
        }

        fn decode_from_array(src: [u8; core::mem::size_of::<$ident>()]) -> Self {
          $ident::from_le_bytes(src)
        }
      }
    )+
  };
}

impl_fixed_value_size!(
  u8::{ |buf: &[u8]| buf[0] },
  u16::{ |buf: &[u8]| u16::from_le_bytes([buf[0], buf[1]]) },
);

impl sealed::Sealed for u32 {
  type Error = VarintError;

  const MAX_SIZE: usize = u32::MAX as usize;
  const MAX_ENCODED_SIZE: usize = 5;

  fn encoded_size(&self) -> usize {
    encoded_len_varint(*self as u64)
  }

  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    encode_varint(*self as u64, buf)
  }

  fn decode(buf: &[u8]) -> Result<(usize, Self), Self::Error> {
    decode_varint(buf).map(|(size, n)| (size, n as u32))
  }
}

impl sealed::Sealed for u64 {
  type Error = VarintError;

  const MAX_SIZE: usize = u64::MAX as usize;
  const MAX_ENCODED_SIZE: usize = 10;

  fn encoded_size(&self) -> usize {
    encoded_len_varint(*self)
  }

  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    encode_varint(*self, buf)
  }

  fn decode(buf: &[u8]) -> Result<(usize, Self), Self::Error> {
    decode_varint(buf)
  }
}

/// Value pointer encode/decode error.
#[derive(Debug, Copy, Clone)]
pub enum ValuePointerError<S> {
  /// Buffer is too small to encode the value pointer.
  BufferTooSmall,
  /// Value size encode/decode error.
  ValueSizeError(S),
  /// Not enough bytes to decode the value pointer.
  NotEnoughBytes,
  /// Returned when encoding/decoding varint failed.
  VarintError(VarintError),
}

impl<S> From<VarintError> for ValuePointerError<S> {
  #[inline]
  fn from(e: VarintError) -> Self {
    Self::VarintError(e)
  }
}

impl<S: core::fmt::Display> core::fmt::Display for ValuePointerError<S> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::BufferTooSmall => write!(f, "encode buffer too small"),
      Self::ValueSizeError(e) => write!(f, "{e}"),
      Self::NotEnoughBytes => write!(f, "not enough bytes"),
      Self::VarintError(e) => write!(f, "{e}"),
    }
  }
}

#[cfg(feature = "std")]
impl<S: core::fmt::Display + core::fmt::Debug> Error for ValuePointerError<S> {}

/// A pointer to the value in the log.
pub struct ValuePointer<S = u32> {
  fid: u32,
  size: S,
  offset: u64,
}

impl<S> ValuePointer<S> {
  /// Returns the id of the file which contains the value.
  #[inline]
  pub const fn fid(&self) -> u32 {
    self.fid
  }

  /// Returns the offset of the value in the file.
  #[inline]
  pub const fn offset(&self) -> u64 {
    self.offset
  }

  /// Returns the size of the value.
  #[inline]
  pub const fn size(&self) -> &S {
    &self.size
  }
}

impl<S: ValueSize> ValuePointer<S> {
  /// Returns the encoded size of the value pointer.
  #[inline]
  pub fn encoded_size(&self) -> usize {
    1 + encoded_len_varint(self.fid as u64) + self.size.encoded_size() + encoded_len_varint(self.offset)
  }

  /// Encodes the value pointer into the buffer.
  pub fn encode(&self, buf: &mut [u8]) -> Result<usize, ValuePointerError<S::Error>> {
    let encoded_size = self.encoded_size();
    if buf.len() < encoded_size {
      return Err(ValuePointerError::BufferTooSmall);
    }

    let mut offset = 0;
    buf[offset] = encoded_size as u8;
    offset += 1;

    offset += encode_varint(self.offset, &mut buf[offset..])?;
    offset += self.size.encode(&mut buf[offset..]).map_err(ValuePointerError::ValueSizeError)?;
    offset += encode_varint(self.fid as u64, &mut buf[offset..])?;

    debug_assert_eq!(encoded_size, offset, "expected encoded size {} is not equal to actual encoded size {}", encoded_size, offset);
    Ok(offset)
  }

  /// Decodes the value pointer from the buffer.
  pub fn decode(buf: &[u8]) -> Result<(usize, Self), ValuePointerError<S::Error>> {
    if buf.is_empty() {
      return Err(ValuePointerError::NotEnoughBytes);
    }

    let encoded_size = buf[0] as usize;
    if buf.len() < encoded_size {
      return Err(ValuePointerError::NotEnoughBytes);
    }

    let mut cur = 1;
    let (read, fid) = decode_varint(&buf[cur..])?;
    cur += read;
    let (read, size) = S::decode(&buf[cur..]).map_err(ValuePointerError::ValueSizeError)?;
    cur += read;
    let (read, offset) = decode_varint(&buf[cur..])?;
    cur += read;
    debug_assert_eq!(encoded_size, cur, "expected read {} bytes is not equal to actual read bytes {}", encoded_size, cur);

    Ok((encoded_size, Self { fid: fid as u32, size, offset }))
  }
}

mod sync;
pub use sync::*;
use virtualfs::utils::{decode_varint, encode_varint, encoded_len_varint, VarintError};

/// Asynchronous version of the key-value seperated WAL log
#[cfg(feature = "future")]
#[cfg_attr(docsrs, doc(cfg(feature = "future")))]
pub mod future;
