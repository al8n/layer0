use {
  super::types::{KeyRef, Type},
  cheap_clone::CheapClone,
  core::{
    borrow::Borrow,
    cmp::{self, Ordering, Reverse},
    marker::PhantomData,
    ops::{Bound, RangeBounds},
  },
};

/// Custom bytes equivalence trait.
pub trait Equivalentor: core::fmt::Debug {
  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent(&self, a: &[u8], b: &[u8]) -> bool;
}

/// Custom bytes ordering trait.
pub trait Comparator: Equivalentor {
  /// Compare `a` to `b` and return their ordering.
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering;
}

/// `RangeComparator` is implemented as an extention to `Comparator` to
/// allow for comparison of items with range bounds.
pub trait RangeComparator: Comparator {
  /// Returns `true` if `item` is contained in the range.
  fn compare_contains<R, Q>(&self, range: R, item: &[u8]) -> bool
  where
    Q: ?Sized + Borrow<[u8]>,
    R: RangeBounds<Q>,
  {
    let start = match range.start_bound() {
      Bound::Included(start) => self.compare(item, start.borrow()) != Ordering::Less,
      Bound::Excluded(start) => self.compare(item, start.borrow()) == Ordering::Greater,
      Bound::Unbounded => true,
    };

    let end = match range.end_bound() {
      Bound::Included(end) => self.compare(item, end.borrow()) != Ordering::Greater,
      Bound::Excluded(end) => self.compare(item, end.borrow()) == Ordering::Less,
      Bound::Unbounded => true,
    };

    start && end
  }
}

impl<C> RangeComparator for C where C: Comparator {}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  macro_rules! impl_traits {
    ($($ty:ty),+$(,)?) => {
      $(
        impl<C: Equivalentor> Equivalentor for $ty {
          #[inline]
          fn equivalent(&self, a: &[u8], b: &[u8]) -> bool {
            (**self).equivalent(a, b)
          }
        }

        impl<C: Comparator> Comparator for $ty {
          #[inline]
          fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering {
            (**self).compare(a, b)
          }
        }
      )*
    };
  }

  impl_traits!(std::sync::Arc<C>, std::rc::Rc<C>,);
};

impl<C: StaticEquivalentor> StaticEquivalentor for Reverse<C> {
  #[inline]
  fn equivalent(a: &[u8], b: &[u8]) -> bool {
    C::equivalent(b, a)
  }
}

impl<C: StaticComparator> StaticComparator for Reverse<C> {
  #[inline]
  fn compare(a: &[u8], b: &[u8]) -> cmp::Ordering {
    C::compare(b, a)
  }
}

/// Ascend is a comparator that compares byte slices in ascending order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ascend;

impl StaticEquivalentor for Ascend {
  #[inline]
  fn equivalent(a: &[u8], b: &[u8]) -> bool {
    a == b
  }
}

impl StaticComparator for Ascend {
  #[inline]
  fn compare(a: &[u8], b: &[u8]) -> cmp::Ordering {
    a.cmp(b)
  }
}

impl CheapClone for Ascend {}

/// Descend is a comparator that compares byte slices in descending order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Descend;

impl StaticEquivalentor for Descend {
  #[inline]
  fn equivalent(a: &[u8], b: &[u8]) -> bool {
    a == b
  }
}

impl StaticComparator for Descend {
  #[inline]
  fn compare(a: &[u8], b: &[u8]) -> cmp::Ordering {
    b.cmp(a)
  }
}

impl CheapClone for Descend {}

/// Custom bytes equivalence trait.
///
/// Comparing to [`Equivalentor`], `StaticEquivalentor` is not object-safe, but it does not to create a new object when comparing.
pub trait StaticEquivalentor: core::fmt::Debug {
  /// Compare `a` to `b` and return `true` if they are equal.
  #[inline]
  fn equivalent(a: &[u8], b: &[u8]) -> bool {
    a == b
  }
}

/// Custom bytes ordering trait.
///
/// Comparing to [`Comparator`], `StaticComparator` is not object-safe, but it does not to create a new object when comparing.
pub trait StaticComparator: core::fmt::Debug + StaticEquivalentor {
  /// Compare `a` to `b` and return their ordering.
  fn compare(a: &[u8], b: &[u8]) -> cmp::Ordering;
}

/// `StaticRangeComparator` is implemented as an extention to `StaticComparator` to
/// allow for comparison of items with range bounds.
pub trait StaticRangeComparator: StaticComparator {
  /// Returns `true` if `item` is contained in the range.
  fn compare_contains<R, Q>(range: R, item: &[u8]) -> bool
  where
    Q: ?Sized + Borrow<[u8]>,
    R: RangeBounds<Q>,
  {
    let start = match range.start_bound() {
      Bound::Included(start) => Self::compare(start.borrow(), item) != Ordering::Less,
      Bound::Excluded(start) => Self::compare(start.borrow(), item) == Ordering::Greater,
      Bound::Unbounded => true,
    };

    let end = match range.end_bound() {
      Bound::Included(end) => Self::compare(end.borrow(), item) != Ordering::Greater,
      Bound::Excluded(end) => Self::compare(end.borrow(), item) == Ordering::Less,
      Bound::Unbounded => true,
    };

    start && end
  }
}

impl<C> StaticRangeComparator for C where C: StaticComparator {}

impl<K> StaticEquivalentor for PhantomData<K>
where
  K: Type,
  for<'a> K::Ref<'a>: KeyRef<'a, K>,
{
  #[inline]
  fn equivalent(a: &[u8], b: &[u8]) -> bool {
    unsafe { <K::Ref<'_> as KeyRef<'_, K>>::equivalent_binary(a, b) }
  }
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
}

impl<T: StaticEquivalentor> Equivalentor for T {
  #[inline]
  fn equivalent(&self, a: &[u8], b: &[u8]) -> bool {
    T::equivalent(a, b)
  }
}

impl<T: StaticComparator> Comparator for T {
  #[inline]
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering {
    T::compare(a, b)
  }
}

#[cfg(test)]
mod tests {
  use core::cmp;

  use super::{Ascend, Comparator, Descend, Equivalentor, RangeComparator, Reverse};

  #[test]
  fn test_desc() {
    let desc = Descend;
    assert_eq!(desc.compare(b"abc", b"def"), cmp::Ordering::Greater);
    assert_eq!(desc.compare(b"def", b"abc"), cmp::Ordering::Less);
    assert_eq!(desc.compare(b"abc", b"abc"), cmp::Ordering::Equal);

    assert!(desc.compare_contains::<_, &[u8]>("b".as_bytes().."a".as_bytes(), b"b"));
  }

  #[test]
  fn test_desc_reverse() {
    let desc = Reverse(Descend);
    assert_eq!(desc.compare(b"abc", b"def"), cmp::Ordering::Less);
    assert_eq!(desc.compare(b"def", b"abc"), cmp::Ordering::Greater);
    assert_eq!(desc.compare(b"abc", b"abc"), cmp::Ordering::Equal);

    assert!(desc.compare_contains("a".as_bytes().."d".as_bytes(), b"b"));
  }

  #[test]
  fn test_asc() {
    let asc = Ascend;
    assert_eq!(asc.compare(b"abc", b"def"), cmp::Ordering::Less);
    assert_eq!(asc.compare(b"def", b"abc"), cmp::Ordering::Greater);
    assert_eq!(asc.compare(b"abc", b"abc"), cmp::Ordering::Equal);

    assert!(asc.compare_contains("a".as_bytes().."d".as_bytes(), b"b"));
  }

  #[test]
  fn test_asc_reverse() {
    let asc = Reverse(Ascend);
    assert_eq!(asc.compare(b"abc", b"def"), cmp::Ordering::Greater);
    assert_eq!(asc.compare(b"def", b"abc"), cmp::Ordering::Less);
    assert_eq!(asc.compare(b"abc", b"abc"), cmp::Ordering::Equal);
    assert!(asc.equivalent(b"a", b"a"));
    assert!(asc.compare_contains("d".as_bytes()..="a".as_bytes(), b"d"));
  }

  #[cfg(any(feature = "std", feature = "alloc"))]
  #[test]
  fn test_arc() {
    let arc = std::sync::Arc::new(Ascend);
    assert_eq!(arc.compare(b"abc", b"def"), cmp::Ordering::Less);
    assert_eq!(arc.compare(b"def", b"abc"), cmp::Ordering::Greater);
    assert_eq!(arc.compare(b"abc", b"abc"), cmp::Ordering::Equal);

    assert!(arc.compare_contains("a".as_bytes().."d".as_bytes(), b"b"));

    assert!(arc.equivalent(b"abc", b"abc"));
  }
}
