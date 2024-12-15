use core::cell::OnceCell;

use super::TypeRef;

/// A lazy initialized reference type for a [`Type`](super::Type).
pub struct LazyRef<'a, T> {
  raw: Option<&'a [u8]>,
  val: OnceCell<T>,
}

impl<'a, T> LazyRef<'a, T>
where
  T: TypeRef<'a>,
{
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
  pub fn new(val: T) -> Self {
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
  pub unsafe fn with_raw(val: T, raw: &'a [u8]) -> Self {
    Self {
      raw: Some(raw),
      val: OnceCell::from(val),
    }
  }

  /// Returns the raw byte slice if it exists.
  #[inline]
  pub const fn raw(&self) -> Option<&'a [u8]> {
    self.raw
  }
}

impl<'a, T> LazyRef<'a, T>
where
  T: TypeRef<'a>,
{
  /// Returns the reference value.
  pub fn get(&self) -> &T {
    self.val.get_or_init(|| unsafe {
      T::from_slice(
        self
          .raw
          .expect("value must be initialized when raw is None"),
      )
    })
  }
}

impl<'a, T> core::fmt::Debug for LazyRef<'a, T>
where
  T: core::fmt::Debug + TypeRef<'a>,
{
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_tuple("LazyRef").field(&self.get()).finish()
  }
}

impl<'a, T> core::fmt::Display for LazyRef<'a, T>
where
  T: core::fmt::Display + TypeRef<'a>,
{
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.get())
  }
}

impl<'a, T> PartialEq for LazyRef<'a, T>
where
  T: TypeRef<'a> + PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.get() == other.get()
  }
}

impl<'a, T> Eq for LazyRef<'a, T> where T: TypeRef<'a> + Eq {}

impl<'a, T> PartialOrd for LazyRef<'a, T>
where
  T: TypeRef<'a> + PartialOrd,
{
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    self.get().partial_cmp(other.get())
  }
}

impl<'a, T> Ord for LazyRef<'a, T>
where
  T: TypeRef<'a> + Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    self.get().cmp(other.get())
  }
}

impl<'a, T> core::hash::Hash for LazyRef<'a, T>
where
  T: TypeRef<'a> + core::hash::Hash,
{
  #[inline]
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.get().hash(state)
  }
}

impl<'a, T> Clone for LazyRef<'a, T>
where
  T: TypeRef<'a>,
{
  #[inline]
  fn clone(&self) -> Self {
    Self {
      raw: self.raw,
      val: self.val.clone(),
    }
  }
}

impl<'a, T> AsRef<T> for LazyRef<'a, T>
where
  T: TypeRef<'a>,
{
  #[inline]
  fn as_ref(&self) -> &T {
    self
  }
}

impl<'a, T> core::ops::Deref for LazyRef<'a, T>
where
  T: TypeRef<'a>,
{
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.get()
  }
}
