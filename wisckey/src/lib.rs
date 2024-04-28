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
pub trait ValueSize: FromUsize + IntoUsize + Sized {
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

        impl ValueSize for $ident {
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

impl ValueSize for u32 {
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

impl ValueSize for u64 {
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

pub struct ValuePointer<I, S = u32> {
  fid: I,
  offset: u64,
  size: S,
}

impl<I, S> ValuePointer<I, S> {
  /// Returns the id of the file which contains the value.
  #[inline]
  pub const fn fid(&self) -> &I {
    &self.fid
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

mod sync;
pub use sync::*;
use virtualfs::utils::{decode_varint, encode_varint, encoded_len_varint, VarintError};

/// Asynchronous version of the key-value seperated WAL log
#[cfg(feature = "future")]
#[cfg_attr(docsrs, doc(cfg(feature = "future")))]
pub mod future;
