//! Utils for developing database
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]

#[cfg(any(feature = "std", test))]
extern crate std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

/// Traits and structs for checksuming.
pub mod checksum;

/// LEB128 encoding and decoding
pub mod leb128;

/// A vacant buffer that can be filled with bytes.
pub mod buffer;

/// Some traits may be useful.
pub mod traits;
pub use traits::Comparator;

#[doc(inline)]
pub use equivalent;

pub use cheap_clone::CheapClone;
