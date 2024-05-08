//! Virtual File System
#![allow(clippy::type_complexity)]
// #![deny(warnings, missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![cfg_attr(not(all(feature = "std", test)), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc as std;

/// Utils
pub mod utils;

/// Enumeration of possible methods to seek within an I/O object.
///
/// It is used by the [`Seek`] trait.
#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum SeekFrom {
  /// Sets the offset to the provided number of bytes.
  Start(u64),

  /// Sets the offset to the size of this object plus the specified number of
  /// bytes.
  ///
  /// It is possible to seek beyond the end of an object, but it's an error to
  /// seek before byte 0.
  End(i64),

  /// Sets the offset to the current position plus the specified number of
  /// bytes.
  ///
  /// It is possible to seek beyond the end of an object, but it's an error to
  /// seek before byte 0.
  Current(i64),
}

mod sync;
pub use sync::*;

#[cfg(feature = "future")]
mod future;
#[cfg(feature = "future")]
#[cfg_attr(docsrs, doc(cfg(feature = "future")))]
pub use future::*;
