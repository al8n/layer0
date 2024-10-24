use core::{
  cmp::{self, Reverse},
  marker::PhantomData,
  ops::{Bound, RangeBounds},
};

use cheap_clone::CheapClone;

use super::{KeyRef, Type};

/// StaticComparator is used for key-value database developers to define their own key comparison logic.
/// e.g. some key-value database developers may want to alpabetically comparation.
///
/// Comparing to [`Comparator`], `StaticComparator` is not object-safe, but it does not to create a new object when comparing.
pub trait StaticComparator: core::fmt::Debug {
  /// Compares two byte slices.
  fn compare(a: &[u8], b: &[u8]) -> cmp::Ordering;

  /// Returns if a is contained in range.
  fn contains(start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool;
}

impl<K> StaticComparator for PhantomData<K>
where
  K: Type,
  for<'a> K::Ref<'a>: KeyRef<'a, K>,
{
  #[inline]
  fn compare(a: &[u8], b: &[u8]) -> cmp::Ordering {
    unsafe { <K::Ref<'_> as KeyRef<'_, K>>::compare_binary(a, b) }
  }

  #[inline]
  fn contains(start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool {
    unsafe { <K::Ref<'_> as KeyRef<'_, K>>::contains_binary(start_bound, end_bound, key) }
  }
}

impl<T: StaticComparator> Comparator for T {
  #[inline]
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering {
    T::compare(a, b)
  }

  #[inline]
  fn contains(&self, start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool {
    T::contains(start_bound, end_bound, key)
  }
}

/// Comparator is used for key-value database developers to define their own key comparison logic.
/// e.g. some key-value database developers may want to alpabetically comparation
pub trait Comparator: core::fmt::Debug {
  /// Compares two byte slices.
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering;

  /// Returns if a is contained in range.
  fn contains(&self, start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool;
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<C: Comparator> Comparator for std::sync::Arc<C> {
  #[inline]
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering {
    (**self).compare(a, b)
  }

  fn contains(&self, start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool {
    (**self).contains(start_bound, end_bound, key)
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<C: Comparator> Comparator for std::rc::Rc<C> {
  #[inline]
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering {
    (**self).compare(a, b)
  }

  fn contains(&self, start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool {
    (**self).contains(start_bound, end_bound, key)
  }
}

impl<C: StaticComparator> StaticComparator for Reverse<C> {
  #[inline]
  fn compare(a: &[u8], b: &[u8]) -> cmp::Ordering {
    C::compare(b, a)
  }

  #[inline]
  fn contains(start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool {
    C::contains(start_bound, end_bound, key)
  }
}

/// Ascend is a comparator that compares byte slices in ascending order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ascend;

impl StaticComparator for Ascend {
  #[inline]
  fn compare(a: &[u8], b: &[u8]) -> cmp::Ordering {
    a.cmp(b)
  }

  #[inline]
  fn contains(start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool {
    <(Bound<&[u8]>, Bound<&[u8]>) as RangeBounds<&[u8]>>::contains::<&[u8]>(
      &(start_bound, end_bound),
      &key,
    )
  }
}

impl CheapClone for Ascend {}

/// Descend is a comparator that compares byte slices in descending order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Descend;

impl StaticComparator for Descend {
  #[inline]
  fn compare(a: &[u8], b: &[u8]) -> cmp::Ordering {
    b.cmp(a)
  }

  #[inline]
  fn contains(start_bound: Bound<&[u8]>, end_bound: Bound<&[u8]>, key: &[u8]) -> bool {
    <(Bound<&[u8]>, Bound<&[u8]>) as RangeBounds<&[u8]>>::contains::<&[u8]>(
      &(start_bound, end_bound),
      &key,
    )
  }
}

impl CheapClone for Descend {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_desc() {
    let desc = Descend;
    assert_eq!(desc.compare(b"abc", b"def"), cmp::Ordering::Greater);
    assert_eq!(desc.compare(b"def", b"abc"), cmp::Ordering::Less);
    assert_eq!(desc.compare(b"abc", b"abc"), cmp::Ordering::Equal);

    assert!(desc.contains(Bound::Included(b"a"), Bound::Excluded(b"d"), b"b"));
  }

  #[test]
  fn test_desc_reverse() {
    let desc = Reverse(Descend);
    assert_eq!(desc.compare(b"abc", b"def"), cmp::Ordering::Less);
    assert_eq!(desc.compare(b"def", b"abc"), cmp::Ordering::Greater);
    assert_eq!(desc.compare(b"abc", b"abc"), cmp::Ordering::Equal);

    assert!(desc.contains(Bound::Included(b"a"), Bound::Excluded(b"d"), b"b"));
  }

  #[test]
  fn test_asc() {
    let asc = Ascend;
    assert_eq!(asc.compare(b"abc", b"def"), cmp::Ordering::Less);
    assert_eq!(asc.compare(b"def", b"abc"), cmp::Ordering::Greater);
    assert_eq!(asc.compare(b"abc", b"abc"), cmp::Ordering::Equal);

    assert!(asc.contains(Bound::Included(b"a"), Bound::Excluded(b"d"), b"b"));
  }

  #[test]
  fn test_asc_reverse() {
    let asc = Reverse(Ascend);
    assert_eq!(asc.compare(b"abc", b"def"), cmp::Ordering::Greater);
    assert_eq!(asc.compare(b"def", b"abc"), cmp::Ordering::Less);
    assert_eq!(asc.compare(b"abc", b"abc"), cmp::Ordering::Equal);

    assert!(asc.contains(Bound::Included(b"a"), Bound::Excluded(b"d"), b"b"));
  }
}
