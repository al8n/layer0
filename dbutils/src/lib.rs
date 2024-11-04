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

/// Common error types.
pub mod error;

pub use cheap_clone::CheapClone;

/// Re-exports [`equivalent`](::equivalent) crate and extensions.
pub mod equivalent;

/// Similar to [`equivalent`], but for bytes.
pub mod equivalentor;

/// Types and traits for encoding and decoding.
pub mod types;

#[doc(hidden)]
pub mod __private {
  pub use paste;
}

/// A macro to generate builder types.
///
/// A builder type is typically has a size and a closure that can fill a buffer with the given size.
///
/// ## Example
///
/// ```rust
/// use dbutils::{builder, buffer::VacantBuffer};
///
/// // Generates a builder type named `ValueBuilder` with a maximum size of `u32`.
/// builder!(
///   /// A builder for a value
///   ValueBuilder;
/// );
///
/// let builder = ValueBuilder::new(16, |mut buf: VacantBuffer<'_>| {
///   buf.fill(1);
///   Ok::<_, core::convert::Infallible>(buf.len())
/// });
///
/// assert_eq!(builder.size(), 16);
/// ```
#[macro_export]
macro_rules! builder {
  ($(
    $(#[$meta:meta])*
    $wrapper_vis:vis $name:ident
   ); +$(;)?
  ) => {
    $(
      $crate::__private::paste::paste! {
        $(#[$meta])*
        #[derive(::core::marker::Copy, ::core::clone::Clone, ::core::fmt::Debug)]
        $wrapper_vis struct $name <F> {
          /// The required size of the builder.
          $wrapper_vis size: usize,

          /// The builder closure.
          $wrapper_vis f: F,
        }

        impl<F> ::core::convert::From<(usize, F)> for $name<F> {
          #[inline]
          fn from((size, f): (usize, F)) -> Self {
            Self { size, f }
          }
        }

        impl<F> ::core::convert::From<$name<F>> for (usize, F) {
          #[inline]
          fn from(builder: $name<F>) -> Self {
            (builder.size, builder.f)
          }
        }

        impl<F> $name<F> {
          #[doc = "Creates a new `" $name "` with the given size and builder closure."]
          #[inline]
          pub const fn new(size: usize, f: F) -> Self
          {
            Self { size, f }
          }

          #[doc = "Returns the required `" $name "` size."]
          #[inline]
          pub const fn size(&self) -> usize {
            self.size
          }

          #[doc = "Returns the `" $name "` builder closure."]
          #[inline]
          pub const fn builder(&self) -> &F {
            &self.f
          }

          /// Deconstructs the value builder into the size and the builder closure.
          #[inline]
          pub fn into_components(self) -> (usize, F) {
            (self.size, self.f)
          }
        }

        impl<W, E> $crate::buffer::BufWriter for $name<W>
        where
          W: ::core::ops::Fn(&mut $crate::buffer::VacantBuffer<'_>) -> ::core::result::Result<usize, E>,
        {
          type Error = E;

          #[inline]
          fn encoded_len(&self) -> usize {
            self.size()
          }

          #[inline]
          fn write(&self, buf: &mut $crate::buffer::VacantBuffer<'_>) -> ::core::result::Result<usize, Self::Error> {
            self.builder()(buf)
          }
        }

        impl<W, E> $crate::buffer::BufWriterOnce for $name<W>
        where
          W: ::core::ops::FnOnce(&mut $crate::buffer::VacantBuffer<'_>) -> ::core::result::Result<usize, E>,
        {
          type Error = E;

          #[inline]
          fn encoded_len(&self) -> usize {
            self.size()
          }

          #[inline]
          fn write_once(self, buf: &mut $crate::buffer::VacantBuffer<'_>) -> ::core::result::Result<usize, Self::Error> {
            self.into_components().1(buf)
          }
        }
      }
    )*
  };
}
