#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]
#![deny(warnings, missing_docs)]
#![cfg_attr(not(feature = "sync"), forbid(unsafe_code))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("`waterark` requires either `std` or `alloc` feature be enabled");

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

/// Closer implementations.
#[cfg(any(feature = "future", feature = "sync"))]
pub mod closer;

mod watermark;
pub use watermark::WaterMarkError;

#[cfg(feature = "sync")]
#[cfg_attr(docsrs, doc(cfg(feature = "sync")))]
pub use watermark::sync::{self, WaterMark};

#[cfg(feature = "future")]
#[cfg_attr(docsrs, doc(cfg(feature = "future")))]
pub use watermark::future::{self, AsyncWaterMark};

#[cfg(feature = "future")]
#[cfg_attr(docsrs, doc(cfg(feature = "future")))]
pub use agnostic_lite::{AsyncSpawner, Detach};

#[cfg(all(feature = "tokio", feature = "future"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "tokio", feature = "future"))))]
pub use agnostic_lite::tokio::TokioSpawner;

#[cfg(feature = "async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
pub use agnostic_lite::async_std::AsyncStdSpawner;

#[cfg(feature = "smol")]
#[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
pub use agnostic_lite::smol::SmolSpawner;

#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
pub use agnostic_lite::wasm::WasmSpawner;
