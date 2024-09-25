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
///   ValueBuilder(u32)
/// );
///
/// let builder = ValueBuilder::new(16, |mut buf: VacantBuffer<'_>| {
///   buf.fill(1);
///   Ok::<(), core::convert::Infallible>(())
/// });
///
/// assert_eq!(builder.size(), 16);
/// ```
#[macro_export]
macro_rules! builder {
  ($(
    $(#[$meta:meta])*
    $wrapper_vis:vis $name:ident($vis:vis $size:ident));+ $(;)?
  ) => {
    $(
      $crate::__private::paste::paste! {
        $(#[$meta])*
        #[derive(::core::marker::Copy, ::core::clone::Clone, ::core::fmt::Debug)]
        $wrapper_vis struct $name <F> {
          /// The required size of the builder.
          $vis size: $size,
          /// The builder closure.
          $vis f: F,
        }

        impl<F> $name<F> {
          #[doc = "Creates a new `" $name "` with the given size and builder closure."]
          #[inline]
          pub const fn new(size: $size, f: F) -> Self
          {
            Self { size, f }
          }

          #[doc = "Returns the required `" $name "` size."]
          #[inline]
          pub const fn size(&self) -> $size {
            self.size
          }

          #[doc = "Returns the `" $name "` builder closure."]
          #[inline]
          pub const fn builder(&self) -> &F {
            &self.f
          }

          /// Deconstructs the value builder into the size and the builder closure.
          #[inline]
          pub fn into_components(self) -> ($size, F) {
            (self.size, self.f)
          }
        }
      }
    )*
  };
}
