// Copyright 2013 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.
#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

#[cfg(any(feature = "std", feature = "alloc", test))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
mod filter;
#[cfg(any(feature = "std", feature = "alloc", test))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub use filter::{bits_per_key, Filter};

mod frozen_filter;
pub use frozen_filter::FrozenFilter;

/// Hashers for bloomfilter.
pub mod hasher;
pub use hasher::BloomHasher;
