use core::cell::OnceCell;

use super::{Type, TypeRef};

/// A lazy initialized reference type for a [`Type`].
pub struct LazyRef<'a, T: ?Sized + Type> {
  raw: Option<&'a [u8]>,
  val: OnceCell<T::Ref<'a>>,
}

impl<'a, T: ?Sized + Type> LazyRef<'a, T> {
  /// Creates a new `LazyRef` from a raw byte slice.
  ///
  /// ## Safety
  /// - The raw byte slice must be valid for decoding by [`TypeRef::from_slice`].
  #[inline]
  pub const unsafe fn from_raw(raw: &'a [u8]) -> Self {
    Self {
      raw: Some(raw),
      val: OnceCell::new(),
    }
  }

  /// Creates a new `LazyRef` from an initialized reference type.
  #[inline]
  pub fn new(val: T::Ref<'a>) -> Self {
    Self {
      raw: val.as_raw(),
      val: OnceCell::from(val),
    }
  }

  /// Creates a new `LazyRef` from an initialized reference type and its raw byte slice.
  ///
  /// ## Safety
  /// - The result of [`TypeRef::from_slice(raw)`](TypeRef::from_slice) must be the same as the given `val`.
  #[inline]
  pub unsafe fn with_raw(val: T::Ref<'a>, raw: &'a [u8]) -> Self {
    Self {
      raw: Some(raw),
      val: OnceCell::from(val),
    }
  }

  /// Returns the reference value.
  pub fn get(&self) -> &T::Ref<'a> {
    self.val.get_or_init(|| unsafe {
      T::Ref::from_slice(
        self
          .raw
          .expect("value must be initialized when raw is None"),
      )
    })
  }

  /// Returns the raw byte slice if it exists.
  #[inline]
  pub const fn raw(&self) -> Option<&'a [u8]> {
    self.raw
  }
}

impl<'a, T> core::fmt::Debug for LazyRef<'a, T>
where
  T: ?Sized + Type,
  T::Ref<'a>: core::fmt::Debug,
{
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_tuple("LazyRef").field(&self.get()).finish()
  }
}

impl<'a, T> core::fmt::Display for LazyRef<'a, T>
where
  T: ?Sized + Type,
  T::Ref<'a>: core::fmt::Display,
{
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.get())
  }
}

impl<'a, T> PartialEq for LazyRef<'a, T>
where
  T: ?Sized + Type,
  T::Ref<'a>: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.raw == other.raw
  }
}

impl<'a, T> Eq for LazyRef<'a, T>
where
  T: ?Sized + Type,
  T::Ref<'a>: Eq,
{
}

impl<'a, T> PartialOrd for LazyRef<'a, T>
where
  T: ?Sized + Type,
  T::Ref<'a>: PartialOrd,
{
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    self.get().partial_cmp(other.get())
  }
}

impl<'a, T> Ord for LazyRef<'a, T>
where
  T: ?Sized + Type,
  T::Ref<'a>: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    self.get().cmp(other.get())
  }
}

impl<'a, T> core::hash::Hash for LazyRef<'a, T>
where
  T: ?Sized + Type,
  T::Ref<'a>: core::hash::Hash,
{
  #[inline]
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.get().hash(state)
  }
}

impl<'a, T> Clone for LazyRef<'a, T>
where
  T: ?Sized + Type,
  T::Ref<'a>: Clone,
{
  #[inline]
  fn clone(&self) -> Self {
    Self {
      raw: self.raw,
      val: self.val.clone(),
    }
  }
}

impl<'a, T> AsRef<T::Ref<'a>> for LazyRef<'a, T>
where
  T: ?Sized + Type,
{
  #[inline]
  fn as_ref(&self) -> &T::Ref<'a> {
    self
  }
}

impl<'a, T> core::ops::Deref for LazyRef<'a, T>
where
  T: ?Sized + Type,
{
  type Target = T::Ref<'a>;

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.get()
  }
}
