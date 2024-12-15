use core::marker::PhantomData;

use crate::types::{LazyRef, Type, TypeRef};

// /// State trait
// pub trait State<'a> {
//   /// The output type of this state.
//   type Output: ?Sized;

//   /// Returns the output of this state.
//   fn output(&self) -> Self::Output;
// }

// impl<'a> State<'a> for &'a [u8] {
//   type Output = Self;

//   #[inline]
//   fn output(&self) -> Self::Output {
//     self
//   }
// }

// impl<'a, T> State<'a> for LazyRef<'a, T>
// where
//   T: TypeRef<'a>,
// {
//   type Output = T;

//   #[inline]
//   fn output(&self) -> Self::Output {
//     *self.get()
//   }
// }

// impl<'a> State<'a> for Option<&'a [u8]> {
//   type Output = Self;

//   #[inline]
//   fn output(&self) -> Self::Output {
//     self.as_ref().copied()
//   }
// }

// impl<'a, T> State<'a> for Option<LazyRef<'a, T>>
// where
//   T: TypeRef<'a>,
// {
//   type Output = Option<T>;

//   #[inline]
//   fn output(&self) -> Self::Output {
//     self.as_ref().map(|v| *v.get())
//   }
// }

/// The state for the entry.
pub trait State<'a> {
  /// The data type of this state.
  type Data: ?Sized;
}

/// A state for the entry that is active.
pub struct Active<T: ?Sized>(PhantomData<T>);

impl<'a, T: ?Sized> State<'a> for Active<T> {
  type Data = T;
}

/// A state for the entry that may be a tombstone.
pub struct MaybeTombstone<T: ?Sized>(PhantomData<T>);

impl<'a, T> State<'a> for MaybeTombstone<T> {
  type Data = Option<T>;
}

// /// Mode
// pub trait Mode {}

// /// Type are read and written as bytes.
// pub struct Bytes;

// /// Type are read and written as generic.
// pub struct Generic;

// /// a
// pub struct Active<M: ?Sized>(PhantomData<M>);

// impl<'a> State<'a, [u8]> for Active<Bytes> {
//   // type Value<V> = &'a [u8] where V: 'a;
//   type Output = &'a [u8];
// }

// impl<'a, V> State<'a, V> for Active<Generic>
// {
//   type Output = LazyRef<'a, V>;
// }

// /// a
// pub struct MaybeTombstone<M: ?Sized>(PhantomData<M>);

// impl<'a> State<'a, [u8]> for MaybeTombstone<Bytes> {
//   type Output = Option<&'a [u8]>;
// }

// impl<'a, V> State<'a, V> for MaybeTombstone<Generic>
// {
//   type Output = Option<LazyRef<'a, V>>;
// }
