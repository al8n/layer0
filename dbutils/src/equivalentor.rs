use {
  cheap_clone::CheapClone,
  core::{
    borrow::Borrow,
    cmp::{self, Ordering, Reverse},
    ops::{Bound, RangeBounds},
  },
  equivalent::{Comparable, Equivalent},
};

mod type_equivalentor;
pub use type_equivalentor::*;

/// Custom equivalence trait.
pub trait DynEquivalentor<A, B>
where
  A: ?Sized,
  B: ?Sized,
{
  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent(&self, a: &A, b: &B) -> bool;
}

/// Custom ordering trait.
pub trait DynComparator<A, B>: DynEquivalentor<A, B>
where
  A: ?Sized,
  B: ?Sized,
{
  /// Compare `a` to `b` and return their ordering.
  fn compare(&self, a: &A, b: &B) -> cmp::Ordering;
}

/// `RangeComparator` is implemented as an extention to `Comparator` to
/// allow for comparison of items with range bounds.
pub trait DynRangeComparator<A, B>: DynComparator<A, B>
where
  A: ?Sized,
  B: ?Sized,
{
  /// Returns `true` if `item` is contained in the range.
  fn compare_contains<R, Q>(&self, range: &R, item: &A) -> bool
  where
    Q: ?Sized + Borrow<B>,
    R: ?Sized + RangeBounds<Q>,
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

impl<A, B, C> DynRangeComparator<A, B> for C
where
  A: ?Sized,
  B: ?Sized,
  C: DynComparator<A, B>,
{
}

/// Custom equivalence trait.
///
/// Comparing to [`DynEquivalentor`], `Equivalentor` is not object-safe, but it can store some information about how to compare.
pub trait Equivalentor {
  /// The fixed type for comparison.
  type A: ?Sized;

  // /// Compare `a` to `b` and return `true` if they are equal.
  // fn query_equivalent(&self, a: &Self::A, b: &Q) -> bool;

  /// Compare `a1` to `a2` and return `true` if they are equal.
  fn equivalent(&self, a1: &Self::A, a2: &Self::A) -> bool;
}

/// Custom equivalence trait for query purpose.
pub trait QueryEquivalentor<Q: ?Sized>: Equivalentor {
  /// Compare `a` to `b` and return `true` if they are equal.
  fn query_equivalent(&self, a: &Self::A, b: &Q) -> bool;
}

impl<C: Equivalentor> QueryEquivalentor<C::A> for C {
  #[inline]
  fn query_equivalent(&self, a: &Self::A, b: &C::A) -> bool {
    self.equivalent(a, b)
  }
}

/// Custom ordering trait.
///
/// Comparing to [`DynComparator`], `Comparator` is not object-safe, but it can store some information about how to compare.
pub trait Comparator: Equivalentor {
  /// Compare `a1` to `a2` and return their ordering.
  fn compare(&self, a1: &Self::A, a2: &Self::A) -> cmp::Ordering;
}

/// Custom equivalence trait for query purpose.
pub trait QueryComparator<Q: ?Sized>: QueryEquivalentor<Q> + Comparator {
  /// Compare `a` to `b` and return their ordering.
  fn query_compare(&self, a: &Self::A, b: &Q) -> cmp::Ordering;
}

impl<C: Comparator> QueryComparator<C::A> for C {
  #[inline]
  fn query_compare(&self, a: &Self::A, b: &C::A) -> cmp::Ordering {
    self.compare(a, b)
  }
}

/// `RangeComparator` is implemented as an extention to `Comparator` to
/// allow for comparison of items with range bounds.
pub trait RangeComparator: Comparator {
  /// Returns `true` if `item` is contained in the range.
  fn compare_contains<Q, R>(&self, range: &R, item: &Self::A) -> bool
  where
    Q: ?Sized + Borrow<Self::A>,
    R: ?Sized + RangeBounds<Q>,
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

/// `RangeComparator` is implemented as an extention to `Comparator` to
/// allow for comparison of items with range bounds.
pub trait QueryRangeComparator<Q: ?Sized>: QueryComparator<Q> {
  /// Returns `true` if `item` is contained in the range.
  #[inline]
  fn query_compare_contains<R>(&self, range: &R, item: &Self::A) -> bool
  where
    R: ?Sized + RangeBounds<Q>,
  {
    let start = match range.start_bound() {
      Bound::Included(start) => self.query_compare(item, start) != Ordering::Less,
      Bound::Excluded(start) => self.query_compare(item, start) == Ordering::Greater,
      Bound::Unbounded => true,
    };

    let end = match range.end_bound() {
      Bound::Included(end) => self.query_compare(item, end) != Ordering::Greater,
      Bound::Excluded(end) => self.query_compare(item, end) == Ordering::Less,
      Bound::Unbounded => true,
    };

    start && end
  }
}

impl<Q: ?Sized, C> QueryRangeComparator<Q> for C where C: QueryComparator<Q> {}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  macro_rules! impl_traits {
    ($($ty:ty),+$(,)?) => {
      $(
        impl<C> Equivalentor for $ty
        where
          C: Equivalentor,
        {
          type A = C::A;

          #[inline]
          fn equivalent(&self, a: &Self::A, b: &Self::A) -> bool
          {
            (**self).equivalent(a, b)
          }
        }

        impl<C> Comparator for $ty
        where
          C: Comparator,
        {
          #[inline]
          fn compare(&self, a: &Self::A, b: &Self::A) -> cmp::Ordering
          {
            (**self).compare(a, b)
          }
        }
      )*
    };
  }

  impl_traits!(std::sync::Arc<C>, std::rc::Rc<C>, std::boxed::Box<C>);

  #[cfg(feature = "triomphe01")]
  impl_traits!(triomphe01::Arc<C>);
};

/// Custom equivalence trait.
///
/// Comparing to [`DynEquivalentor`], `StaticEquivalentor` is not object-safe, but it does not to create a new object when comparing.
pub trait StaticEquivalentor<A, B>
where
  A: ?Sized,
  B: ?Sized,
{
  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent(a: &A, b: &B) -> bool;
}

/// Custom ordering trait.
///
/// Comparing to [`DynComparator`], `StaticComparator` is not object-safe, but it does not to create a new object when comparing.
pub trait StaticComparator<A, B>: StaticEquivalentor<A, B>
where
  A: ?Sized,
  B: ?Sized,
{
  /// Compare `a` to `b` and return their ordering.
  fn compare(a: &A, b: &B) -> cmp::Ordering;
}

/// `StaticRangeComparator` is implemented as an extention to `StaticComparator` to
/// allow for comparison of items with range bounds.
pub trait StaticRangeComparator<A, B>: StaticComparator<A, B>
where
  A: ?Sized,
  B: ?Sized,
{
  /// Returns `true` if `item` is contained in the range.
  fn compare_contains<R, Q>(range: &R, item: &A) -> bool
  where
    Q: ?Sized + Borrow<B>,
    R: ?Sized + RangeBounds<Q>,
  {
    let start = match range.start_bound() {
      Bound::Included(start) => Self::compare(item, start.borrow()) != Ordering::Less,
      Bound::Excluded(start) => Self::compare(item, start.borrow()) == Ordering::Greater,
      Bound::Unbounded => true,
    };

    let end = match range.end_bound() {
      Bound::Included(end) => Self::compare(item, end.borrow()) != Ordering::Greater,
      Bound::Excluded(end) => Self::compare(item, end.borrow()) == Ordering::Less,
      Bound::Unbounded => true,
    };

    start && end
  }
}

impl<A, B, C> StaticRangeComparator<A, B> for C
where
  A: ?Sized,
  B: ?Sized,
  C: StaticComparator<A, B>,
{
}

impl<A, B, C> DynEquivalentor<A, B> for C
where
  A: ?Sized,
  B: ?Sized,
  C: StaticEquivalentor<A, B>,
{
  #[inline]
  fn equivalent(&self, a: &A, b: &B) -> bool {
    C::equivalent(a, b)
  }
}

impl<A, B, C> DynComparator<A, B> for C
where
  A: ?Sized,
  B: ?Sized,
  C: StaticComparator<A, B>,
{
  #[inline]
  fn compare(&self, a: &A, b: &B) -> cmp::Ordering {
    C::compare(a, b)
  }
}

/// Ascend is a comparator that compares byte slices in ascending order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ascend;

impl<A, B> StaticEquivalentor<A, B> for Ascend
where
  A: ?Sized,
  B: ?Sized + Equivalent<A>,
{
  #[inline]
  fn equivalent(a: &A, b: &B) -> bool {
    Equivalent::equivalent(b, a)
  }
}

impl<A, B> StaticComparator<A, B> for Ascend
where
  A: ?Sized,
  B: ?Sized + Comparable<A>,
{
  #[inline]
  fn compare(a: &A, b: &B) -> cmp::Ordering {
    Comparable::compare(b, a).reverse()
  }
}

impl CheapClone for Ascend {}

/// Descend is a comparator that compares byte slices in descending order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Descend;

impl<A, B> StaticEquivalentor<A, B> for Descend
where
  A: ?Sized,
  B: ?Sized + Equivalent<A>,
{
  #[inline]
  fn equivalent(a: &A, b: &B) -> bool {
    Equivalent::equivalent(b, a)
  }
}

impl<A, B> StaticComparator<A, B> for Descend
where
  A: ?Sized,
  B: ?Sized + Comparable<A>,
{
  #[inline]
  fn compare(a: &A, b: &B) -> cmp::Ordering {
    Comparable::compare(b, a)
  }
}

impl CheapClone for Descend {}

impl<A, B, C> StaticEquivalentor<A, B> for Reverse<C>
where
  A: ?Sized,
  B: ?Sized,
  C: StaticEquivalentor<A, B>,
{
  #[inline]
  fn equivalent(a: &A, b: &B) -> bool {
    C::equivalent(a, b)
  }
}

impl<A, B, C> StaticComparator<A, B> for Reverse<C>
where
  A: ?Sized,
  B: ?Sized + Comparable<A>,
  C: StaticComparator<A, B>,
{
  #[inline]
  fn compare(a: &A, b: &B) -> cmp::Ordering {
    C::compare(a, b).reverse()
  }
}

#[cfg(test)]
mod tests {
  use core::cmp;

  use super::{Ascend, Descend, DynComparator, DynEquivalentor, DynRangeComparator, Reverse};

  #[test]
  fn test_desc() {
    let desc = Descend;
    assert_eq!(desc.compare(b"abc", b"def"), cmp::Ordering::Greater);
    assert_eq!(desc.compare(b"def", b"abc"), cmp::Ordering::Less);
    assert_eq!(desc.compare(b"abc", b"abc"), cmp::Ordering::Equal);
    assert!(DynRangeComparator::<[u8], [u8]>::compare_contains(
      &desc,
      &("b".as_bytes().."a".as_bytes()),
      "b".as_bytes()
    ));
  }

  #[test]
  fn test_desc_reverse() {
    let desc = Reverse(Descend);
    assert_eq!(desc.compare(b"abc", b"def"), cmp::Ordering::Less);
    assert_eq!(desc.compare(b"def", b"abc"), cmp::Ordering::Greater);
    assert_eq!(desc.compare(b"abc", b"abc"), cmp::Ordering::Equal);

    assert!(DynRangeComparator::<[u8], [u8]>::compare_contains(
      &desc,
      &("a".as_bytes()..="d".as_bytes()),
      "b".as_bytes()
    ));
  }

  #[test]
  fn test_asc() {
    let asc = Ascend;
    assert_eq!(asc.compare(b"abc", b"def"), cmp::Ordering::Less);
    assert_eq!(asc.compare(b"def", b"abc"), cmp::Ordering::Greater);
    assert_eq!(asc.compare(b"abc", b"abc"), cmp::Ordering::Equal);

    assert!(DynRangeComparator::<[u8], [u8]>::compare_contains(
      &asc,
      &("a".as_bytes().."d".as_bytes()),
      "b".as_bytes()
    ));
  }

  #[test]
  fn test_asc_reverse() {
    let asc = Reverse(Ascend);
    assert_eq!(asc.compare(b"abc", b"def"), cmp::Ordering::Greater);
    assert_eq!(asc.compare(b"def", b"abc"), cmp::Ordering::Less);
    assert_eq!(asc.compare(b"abc", b"abc"), cmp::Ordering::Equal);
    assert!(asc.equivalent(b"a", b"a"));
    assert!(DynRangeComparator::<[u8], [u8]>::compare_contains(
      &asc,
      &("d".as_bytes()..="a".as_bytes()),
      "d".as_bytes()
    ));
  }
}
