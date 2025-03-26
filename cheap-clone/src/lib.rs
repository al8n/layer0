// `CheapClone` trait is inspired by https://github.com/graphprotocol/graph-node/blob/master/graph/src/cheap_clone.rs

//! A trait which indicates that such type can be cloned cheaply.
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

#[cfg(any(feature = "std", test))]
extern crate std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

macro_rules! impl_cheap_clone_for_copy {
  ($($ty: ty), +$(,)?) => {
    $(
      impl crate::CheapClone for $ty {
        fn cheap_clone(&self) -> Self {
          *self
        }
      }
    )*
  };
}

/// Things that are fast to clone in the context of an application.
///
/// The purpose of this API is to reduce the number of calls to .clone() which need to
/// be audited for performance.
///
/// As a rule of thumb, only constant-time `Clone` impls should also implement CheapClone.
/// Eg:
/// - ✔ [`Arc<T>`](std::sync::Arc)
/// - ✔ [`Rc<T>`](std::rc::Rc)
/// - ✔ [`Bytes`](bytes_1::Bytes)
/// - ✗ [`Vec<T>`](std::vec::Vec)
/// - ✔ [`SmolStr`](smol_str03::SmolStr)
/// - ✔ [`FastStr`](faststr02::FastStr)
/// - ✗ [`String`]
pub trait CheapClone: Clone {
  /// Returns a copy of the value.
  fn cheap_clone(&self) -> Self {
    self.clone()
  }
}

#[cfg(feature = "bytes_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
impl CheapClone for bytes_1::Bytes {}

#[cfg(feature = "smol_str_0_3")]
#[cfg_attr(docsrs, doc(cfg(feature = "smol_str_0_3")))]
impl CheapClone for smol_str_0_3::SmolStr {}

#[cfg(feature = "smol_str_0_2")]
#[cfg_attr(docsrs, doc(cfg(feature = "smol_str_0_2")))]
impl CheapClone for smol_str_0_2::SmolStr {}

#[cfg(feature = "faststr_0_2")]
#[cfg_attr(docsrs, doc(cfg(feature = "faststr_0_2")))]
impl CheapClone for faststr_0_2::FastStr {}

#[cfg(feature = "triomphe_0_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "triomphe_0_1")))]
impl<T> CheapClone for triomphe_0_1::Arc<T> {}

#[cfg(any(feature = "alloc", feature = "std"))]
mod a {
  use super::CheapClone;

  impl<T: ?Sized> CheapClone for std::rc::Rc<T> {}
  impl<T: ?Sized> CheapClone for std::sync::Arc<T> {}
}

#[cfg(feature = "std")]
mod s {
  use super::CheapClone;

  impl<T: CheapClone> CheapClone for std::pin::Pin<T> {}

  impl_cheap_clone_for_copy!(
    std::net::IpAddr,
    std::net::Ipv4Addr,
    std::net::Ipv6Addr,
    std::net::SocketAddr,
    std::net::SocketAddrV4,
    std::net::SocketAddrV6,
  );
}

impl<T: CheapClone> CheapClone for core::cmp::Reverse<T> {
  #[inline]
  fn cheap_clone(&self) -> Self {
    core::cmp::Reverse(self.0.cheap_clone())
  }
}
impl<T: CheapClone> CheapClone for Option<T> {
  #[inline]
  fn cheap_clone(&self) -> Self {
    self.as_ref().map(CheapClone::cheap_clone)
  }
}
impl<T: CheapClone, E: CheapClone> CheapClone for Result<T, E> {
  #[inline]
  fn cheap_clone(&self) -> Self {
    match self {
      Ok(ok) => Ok(ok.cheap_clone()),
      Err(err) => Err(err.cheap_clone()),
    }
  }
}
#[cfg(feature = "either")]
impl<L: CheapClone, R: CheapClone> CheapClone for either::Either<L, R> {
  #[inline]
  fn cheap_clone(&self) -> Self {
    match self {
      either::Either::Left(left) => either::Either::Left(left.cheap_clone()),
      either::Either::Right(right) => either::Either::Right(right.cheap_clone()),
    }
  }
}
#[cfg(feature = "among")]
impl<L: CheapClone, M: CheapClone, R: CheapClone> CheapClone for among::Among<L, M, R> {
  #[inline]
  fn cheap_clone(&self) -> Self {
    match self {
      among::Among::Left(left) => among::Among::Left(left.cheap_clone()),
      among::Among::Middle(middle) => among::Among::Middle(middle.cheap_clone()),
      among::Among::Right(right) => among::Among::Right(right.cheap_clone()),
    }
  }
}

impl_cheap_clone_for_copy! {
  (),
  bool, char, f32, f64, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize,
  core::num::NonZeroI8,
  core::num::NonZeroI16,
  core::num::NonZeroI32,
  core::num::NonZeroI64,
  core::num::NonZeroI128,
  core::num::NonZeroIsize,
  core::num::NonZeroU8,
  core::num::NonZeroU16,
  core::num::NonZeroU32,
  core::num::NonZeroU64,
  core::num::NonZeroU128,
  core::num::NonZeroUsize,
  &str
}

impl<T: Copy, const N: usize> CheapClone for [T; N] {
  fn cheap_clone(&self) -> Self {
    *self
  }
}

impl<T> CheapClone for &T {
  fn cheap_clone(&self) -> Self {
    self
  }
}

macro_rules! impl_cheap_clone_for_tuple {
  (@output $($param:literal),+ $(,)?) => {
    ::paste::paste! {
      impl<$([< T $param >]: CheapClone),+> CheapClone for ($([< T $param >],)+) {
        fn cheap_clone(&self) -> Self {
          ($(self.$param.cheap_clone(),)+)
        }
      }
    }
  };
  (@mid $($end:literal),+$(,)?) => {
    $(
      seq_macro::seq!(
        N in 0..=$end {
          impl_cheap_clone_for_tuple!(@output #(N,)*);
        }
      );
    )*
  };
  ($end:literal) => {
    seq_macro::seq!(N in 1..=$end {
      impl_cheap_clone_for_tuple!(@mid N);
    });
  };
}

impl_cheap_clone_for_tuple!(@output 0);
impl_cheap_clone_for_tuple!(96);
