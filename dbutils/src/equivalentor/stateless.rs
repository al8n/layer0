use core::{
  cmp::{self, Ordering},
  ops::{Bound, RangeBounds},
};

use crate::types::Type;

/// Stateless equivalence trait.
///
/// Comparing to [`Equivalentor`](super::Equivalentor), `StaticEquivalentor` is not object-safe, but it does not to create a new object when comparing.
pub trait StaticEquivalentor<T>
where
  T: ?Sized,
{
  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent(a: &T, b: &T) -> bool;
}

/// Stateless equivalence trait for query purpose.
pub trait StaticQueryEquivalentor<T, Q>: StaticEquivalentor<T>
where
  T: ?Sized,
  Q: ?Sized,
{
  /// Compare `a` to `b` and return `true` if they are equal.
  fn query_equivalent(a: &T, b: &Q) -> bool;
}

/// Stateless equivalence trait
pub trait StaticTypeRefEquivalentor<'a, T>: StaticEquivalentor<T>
where
  T: Type + ?Sized,
{
  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent_ref(a: &T, b: &T::Ref<'a>) -> bool;

  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent_refs(a: &T::Ref<'a>, b: &T::Ref<'a>) -> bool;
}

/// Stateless equivalence trait for query purpose.
pub trait StaticTypeRefQueryEquivalentor<'a, T, Q>: StaticTypeRefEquivalentor<'a, T>
where
  T: Type + ?Sized,
  Q: ?Sized,
{
  /// Compare `a` to `b` and return `true` if they are equal.
  fn query_equivalent_ref(a: &T::Ref<'a>, b: &Q) -> bool;
}

/// Stateless ordering trait.
///
/// Comparing to [`Comparator`](super::Comparator), `StaticComparator` is not object-safe, but it does not to create a new object when comparing.
pub trait StaticComparator<T>: StaticEquivalentor<T>
where
  T: ?Sized,
{
  /// Compare `a` to `b` and return their ordering.
  fn compare(a: &T, b: &T) -> cmp::Ordering;
}

/// Stateless ordering trait for query purpose.
pub trait StaticQueryComparator<T, Q>: StaticComparator<T> + StaticQueryEquivalentor<T, Q>
where
  T: ?Sized,
  Q: ?Sized,
{
  /// Compare `a` to `b` and return their ordering.
  fn query_compare(a: &T, b: &Q) -> cmp::Ordering;
}

/// Stateless ordering trait.
pub trait StaticTypeRefComparator<'a, T>:
  StaticComparator<T> + StaticTypeRefEquivalentor<'a, T>
where
  T: Type + ?Sized,
{
  /// Compare `a` to `b` and return their ordering.
  fn compare_ref(a: &T, b: &T::Ref<'a>) -> cmp::Ordering;

  /// Compare `a` to `b` and return their ordering.
  fn compare_refs(a: &T::Ref<'a>, b: &T::Ref<'a>) -> cmp::Ordering;
}

/// Stateless custom ordering trait for query purpose.
pub trait StaticTypeRefQueryComparator<'a, T, Q>:
  StaticTypeRefComparator<'a, T> + StaticTypeRefQueryEquivalentor<'a, T, Q>
where
  T: Type + ?Sized,
  Q: ?Sized,
{
  /// Compare `a` to `b` and return their ordering.
  fn query_compare_ref(a: &T::Ref<'a>, b: &Q) -> cmp::Ordering;
}

/// `StaticRangeComparator` is implemented as an extention to [`StaticComparator`] to
/// allow for comparison of items with range bounds.
pub trait StaticRangeComparator<T>: StaticComparator<T>
where
  T: ?Sized,
{
  /// Returns `true` if `item` is contained in the range.
  #[inline]
  fn contains<R>(range: &R, item: &T) -> bool
  where
    R: ?Sized + RangeBounds<T>,
  {
    let start = match range.start_bound() {
      Bound::Included(start) => Self::compare(item, start) != Ordering::Less,
      Bound::Excluded(start) => Self::compare(item, start) == Ordering::Greater,
      Bound::Unbounded => true,
    };

    let end = match range.end_bound() {
      Bound::Included(end) => Self::compare(item, end) != Ordering::Greater,
      Bound::Excluded(end) => Self::compare(item, end) == Ordering::Less,
      Bound::Unbounded => true,
    };

    start && end
  }
}

impl<T, C> StaticRangeComparator<T> for C
where
  C: StaticComparator<T>,
  T: ?Sized,
{
}

/// `StaticTypeRefRangeComparator` is implemented as an extention to [`StaticTypeRefComparator`] to
/// allow for comparison of items with range bounds.
pub trait StaticTypeRefRangeComparator<'a, T>: StaticTypeRefComparator<'a, T>
where
  T: Type + ?Sized,
{
  /// Returns `true` if `item` is contained in the range.
  #[inline]
  fn contains<R>(range: &R, item: &T) -> bool
  where
    R: ?Sized + RangeBounds<T::Ref<'a>>,
  {
    let start = match range.start_bound() {
      Bound::Included(start) => Self::compare_ref(item, start) != Ordering::Less,
      Bound::Excluded(start) => Self::compare_ref(item, start) == Ordering::Greater,
      Bound::Unbounded => true,
    };

    let end = match range.end_bound() {
      Bound::Included(end) => Self::compare_ref(item, end) != Ordering::Greater,
      Bound::Excluded(end) => Self::compare_ref(item, end) == Ordering::Less,
      Bound::Unbounded => true,
    };

    start && end
  }

  /// Returns `true` if `item` is contained in the range.
  #[inline]
  fn refs_contains<R>(range: &R, item: &T::Ref<'a>) -> bool
  where
    R: ?Sized + RangeBounds<T::Ref<'a>>,
  {
    let start = match range.start_bound() {
      Bound::Included(start) => Self::compare_refs(item, start) != Ordering::Less,
      Bound::Excluded(start) => Self::compare_refs(item, start) == Ordering::Greater,
      Bound::Unbounded => true,
    };

    let end = match range.end_bound() {
      Bound::Included(end) => Self::compare_refs(item, end) != Ordering::Greater,
      Bound::Excluded(end) => Self::compare_refs(item, end) == Ordering::Less,
      Bound::Unbounded => true,
    };

    start && end
  }

  /// Returns `true` if `item` is contained in the range.
  #[inline]
  fn ref_contains<R>(range: &R, item: &T::Ref<'a>) -> bool
  where
    R: ?Sized + RangeBounds<T>,
  {
    let start = match range.start_bound() {
      Bound::Included(start) => Self::compare_ref(start, item).is_le(),
      Bound::Excluded(start) => Self::compare_ref(start, item).is_lt(),
      Bound::Unbounded => true,
    };

    let end = match range.end_bound() {
      Bound::Included(end) => Self::compare_ref(end, item).is_ge(),
      Bound::Excluded(end) => Self::compare_ref(end, item).is_gt(),
      Bound::Unbounded => true,
    };

    start && end
  }
}

impl<'a, T, C> StaticTypeRefRangeComparator<'a, T> for C
where
  C: StaticTypeRefComparator<'a, T>,
  T: Type + ?Sized,
{
}

/// Stateless `StaticQueryRangeComparator` is implemented as an extention to `StaticComparator` to
/// allow for comparison of items with range bounds.
pub trait StaticQueryRangeComparator<T, Q>: StaticQueryComparator<T, Q>
where
  T: ?Sized,
  Q: ?Sized,
{
  /// Returns `true` if `item` is contained in the range.
  fn compare_contains<R>(range: &R, item: &T) -> bool
  where
    R: ?Sized + RangeBounds<Q>,
  {
    let start = match range.start_bound() {
      Bound::Included(start) => Self::query_compare(item, start) != Ordering::Less,
      Bound::Excluded(start) => Self::query_compare(item, start) == Ordering::Greater,
      Bound::Unbounded => true,
    };

    let end = match range.end_bound() {
      Bound::Included(end) => Self::query_compare(item, end) != Ordering::Greater,
      Bound::Excluded(end) => Self::query_compare(item, end) == Ordering::Less,
      Bound::Unbounded => true,
    };

    start && end
  }
}

impl<T, Q, C> StaticQueryRangeComparator<T, Q> for C
where
  C: StaticQueryComparator<T, Q>,
  T: ?Sized,
  Q: ?Sized,
{
}

/// Stateless `StaticTypeRefQueryRangeComparator` is implemented as an extention to `StaticTypeRefQueryComparator` to
/// allow for comparison of items with range bounds.
pub trait StaticTypeRefQueryRangeComparator<'a, T, Q>:
  StaticTypeRefQueryComparator<'a, T, Q>
where
  T: Type + ?Sized,
  Q: ?Sized,
{
  /// Returns `true` if `item` is contained in the range.
  fn query_compare_contains<R>(&self, range: &R, item: &T::Ref<'a>) -> bool
  where
    R: ?Sized + RangeBounds<Q>,
  {
    let start = match range.start_bound() {
      Bound::Included(start) => Self::query_compare_ref(item, start) != Ordering::Less,
      Bound::Excluded(start) => Self::query_compare_ref(item, start) == Ordering::Greater,
      Bound::Unbounded => true,
    };

    let end = match range.end_bound() {
      Bound::Included(end) => Self::query_compare_ref(item, end) != Ordering::Greater,
      Bound::Excluded(end) => Self::query_compare_ref(item, end) == Ordering::Less,
      Bound::Unbounded => true,
    };

    start && end
  }
}

impl<'a, T, Q, C> StaticTypeRefQueryRangeComparator<'a, T, Q> for C
where
  C: StaticTypeRefQueryComparator<'a, T, Q>,
  Q: ?Sized,
  T: Type + ?Sized,
{
}
