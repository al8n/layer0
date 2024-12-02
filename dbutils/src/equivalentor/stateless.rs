use core::{
  cmp::{self, Ordering},
  ops::{Bound, RangeBounds},
};

use crate::types::Type;

use super::{
  Comparator, Equivalentor, QueryComparator, QueryEquivalentor, TypeRefComparator,
  TypeRefEquivalentor, TypeRefQueryComparator, TypeRefQueryEquivalentor,
};

pub use reverse::*;
mod reverse;

/// Stateless equivalence trait.
///
/// Comparing to [`DynEquivalentor`], `StaticEquivalentor` is not object-safe, but it does not to create a new object when comparing.
pub trait StaticEquivalentor {
  /// The type
  type Type: ?Sized;

  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent(a: &Self::Type, b: &Self::Type) -> bool;
}

/// Stateless equivalence trait for query purpose.
pub trait StaticQueryEquivalentor<Q: ?Sized>: StaticEquivalentor {
  /// Compare `a` to `b` and return `true` if they are equal.
  fn query_equivalent(a: &Self::Type, b: &Q) -> bool;
}

/// Stateless equivalence trait
pub trait StaticTypeRefEquivalentor<'a>: StaticEquivalentor
where
  Self::Type: Type,
{
  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent_ref(a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> bool;

  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent_refs(a: &<Self::Type as Type>::Ref<'a>, b: &<Self::Type as Type>::Ref<'a>) -> bool;
}

/// Stateless equivalence trait for query purpose.
pub trait StaticTypeRefQueryEquivalentor<'a, Q: ?Sized>: StaticTypeRefEquivalentor<'a>
where
  Self::Type: Type,
{
  /// Compare `a` to `b` and return `true` if they are equal.
  fn query_equivalent_ref(a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> bool;
}

/// Stateless ordering trait.
///
/// Comparing to [`Comparator`], `StaticComparator` is not object-safe, but it does not to create a new object when comparing.
pub trait StaticComparator: StaticEquivalentor {
  /// Compare `a` to `b` and return their ordering.
  fn compare(a: &Self::Type, b: &Self::Type) -> cmp::Ordering;
}

/// Stateless ordering trait for query purpose.
pub trait StaticQueryComparator<Q: ?Sized>: StaticComparator + StaticQueryEquivalentor<Q> {
  /// Compare `a` to `b` and return their ordering.
  fn query_compare(a: &Self::Type, b: &Q) -> cmp::Ordering;
}

/// Stateless ordering trait.
pub trait StaticTypeRefComparator<'a>: StaticComparator + StaticTypeRefEquivalentor<'a>
where
  Self::Type: Type,
{
  /// Compare `a` to `b` and return their ordering.
  fn compare_ref(a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> cmp::Ordering;

  /// Compare `a` to `b` and return their ordering.
  fn compare_refs(
    a: &<Self::Type as Type>::Ref<'a>,
    b: &<Self::Type as Type>::Ref<'a>,
  ) -> cmp::Ordering;
}

/// Stateless custom ordering trait for query purpose.
pub trait StaticTypeRefQueryComparator<'a, Q: ?Sized>:
  StaticTypeRefComparator<'a> + StaticTypeRefQueryEquivalentor<'a, Q>
where
  Self::Type: Type,
{
  /// Compare `a` to `b` and return their ordering.
  fn query_compare_ref(a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> cmp::Ordering;
}

/// Stateless `StaticQueryRangeComparator` is implemented as an extention to `StaticComparator` to
/// allow for comparison of items with range bounds.
pub trait StaticQueryRangeComparator<Q: ?Sized>: StaticQueryComparator<Q> {
  /// Returns `true` if `item` is contained in the range.
  fn compare_contains<R>(range: &R, item: &Self::Type) -> bool
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

impl<Q, C> StaticQueryRangeComparator<Q> for C
where
  C: StaticQueryComparator<Q>,
  Q: ?Sized,
{
}

/// Stateless `StaticTypeRefQueryRangeComparator` is implemented as an extention to `StaticTypeRefQueryComparator` to
/// allow for comparison of items with range bounds.
pub trait StaticTypeRefQueryRangeComparator<'a, Q: ?Sized>:
  StaticTypeRefQueryComparator<'a, Q>
where
  Self::Type: Type,
{
  /// Returns `true` if `item` is contained in the range.
  fn query_compare_contains<R>(&self, range: &R, item: &<Self::Type as Type>::Ref<'a>) -> bool
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

impl<'a, Q: ?Sized, C> StaticTypeRefQueryRangeComparator<'a, Q> for C
where
  C: StaticTypeRefQueryComparator<'a, Q>,
  C::Type: Type,
{
}

impl<C> Equivalentor for C
where
  C: StaticEquivalentor,
{
  type Type = C::Type;

  #[inline]
  fn equivalent(&self, a: &Self::Type, b: &Self::Type) -> bool {
    C::equivalent(a, b)
  }
}

impl<Q, C> QueryEquivalentor<Q> for C
where
  Q: ?Sized,
  C: StaticQueryEquivalentor<Q>,
{
  #[inline]
  fn query_equivalent(&self, a: &Self::Type, b: &Q) -> bool {
    C::query_equivalent(a, b)
  }
}

impl<'a, C> TypeRefEquivalentor<'a> for C
where
  C: StaticTypeRefEquivalentor<'a>,
  Self::Type: Type,
{
  #[inline]
  fn equivalent_ref(&self, a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> bool {
    C::equivalent_ref(a, b)
  }

  #[inline]
  fn equivalent_refs(
    &self,
    a: &<Self::Type as Type>::Ref<'a>,
    b: &<Self::Type as Type>::Ref<'a>,
  ) -> bool {
    C::equivalent_refs(a, b)
  }
}

impl<'a, Q, C> TypeRefQueryEquivalentor<'a, Q> for C
where
  Q: ?Sized,
  C: StaticTypeRefQueryEquivalentor<'a, Q>,
  Self::Type: Type,
{
  #[inline]
  fn query_equivalent_ref(&self, a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> bool {
    C::query_equivalent_ref(a, b)
  }
}

impl<C> Comparator for C
where
  C: StaticComparator,
{
  #[inline]
  fn compare(&self, a: &Self::Type, b: &Self::Type) -> cmp::Ordering {
    C::compare(a, b)
  }
}

impl<Q, C> QueryComparator<Q> for C
where
  Q: ?Sized,
  C: StaticQueryComparator<Q>,
{
  #[inline]
  fn query_compare(&self, a: &Self::Type, b: &Q) -> cmp::Ordering {
    C::query_compare(a, b)
  }
}

impl<'a, C> TypeRefComparator<'a> for C
where
  C: StaticTypeRefComparator<'a>,
  Self::Type: Type,
{
  #[inline]
  fn compare_ref(&self, a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> cmp::Ordering {
    C::compare_ref(a, b)
  }

  #[inline]
  fn compare_refs(
    &self,
    a: &<Self::Type as Type>::Ref<'a>,
    b: &<Self::Type as Type>::Ref<'a>,
  ) -> cmp::Ordering {
    C::compare_refs(a, b)
  }
}

impl<'a, Q, C> TypeRefQueryComparator<'a, Q> for C
where
  Q: ?Sized,
  C: StaticTypeRefQueryComparator<'a, Q>,
  Self::Type: Type,
{
  #[inline]
  fn query_compare_ref(&self, a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> cmp::Ordering {
    C::query_compare_ref(a, b)
  }
}
