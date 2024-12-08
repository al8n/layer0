use core::{
  cmp::{self, Ordering},
  ops::{Bound, RangeBounds},
};

use crate::types::Type;

mod reverse;

/// Statefull custom equivalence trait.
pub trait Equivalentor<T: ?Sized> {
  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent(&self, a: &T, b: &T) -> bool;
}

/// Statefull custom equivalence trait.
pub trait TypeRefEquivalentor<'a, T>: Equivalentor<T>
where
  T: Type + ?Sized,
{
  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent_ref(&self, a: &T, b: &T::Ref<'a>) -> bool;

  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent_refs(&self, a: &T::Ref<'a>, b: &T::Ref<'a>) -> bool;
}

/// Statefull custom equivalence trait for query purpose.
pub trait TypeRefQueryEquivalentor<'a, T, Q>: TypeRefEquivalentor<'a, T>
where
  Q: ?Sized,
  T: Type + ?Sized,
{
  /// Compare `a` to `b` and return `true` if they are equal.
  fn query_equivalent_ref(&self, a: &T::Ref<'a>, b: &Q) -> bool;
}

/// Statefull custom equivalence trait for query purpose.
pub trait QueryEquivalentor<T, Q>: Equivalentor<T>
where
  T: ?Sized,
  Q: ?Sized,
{
  /// Compare `a` to `b` and return `true` if they are equal.
  fn query_equivalent(&self, a: &T, b: &Q) -> bool;
}

/// Statefull custom ordering trait.
pub trait Comparator<T: ?Sized>: Equivalentor<T> {
  /// Compare `a` to `b` and return their ordering.
  fn compare(&self, a: &T, b: &T) -> cmp::Ordering;
}

/// Statefull custom ordering trait.
pub trait TypeRefComparator<'a, T>: Comparator<T> + TypeRefEquivalentor<'a, T>
where
  T: Type + ?Sized,
{
  /// Compare `a` to `b` and return their ordering.
  fn compare_ref(&self, a: &T, b: &T::Ref<'a>) -> cmp::Ordering;

  /// Compare `a` to `b` and return their ordering.
  fn compare_refs(&self, a: &T::Ref<'a>, b: &T::Ref<'a>) -> cmp::Ordering;
}

/// Statefull custom ordering trait for querying purpose.
pub trait TypeRefQueryComparator<'a, T, Q>:
  TypeRefComparator<'a, T> + TypeRefQueryEquivalentor<'a, T, Q>
where
  T: Type + ?Sized,
  Q: ?Sized,
{
  /// Compare `a` to `b` and return their ordering.
  fn query_compare_ref(&self, a: &T::Ref<'a>, b: &Q) -> cmp::Ordering;
}

/// Statefull custom ordering trait for querying purpose.
pub trait QueryComparator<T, Q>: Comparator<T> + QueryEquivalentor<T, Q>
where
  T: ?Sized,
  Q: ?Sized,
{
  /// Compare `a` to `b` and return their ordering.
  fn query_compare(&self, a: &T, b: &Q) -> cmp::Ordering;
}

/// `TypeRefQueryRangeComparator` is implemented as an extention to [`TypeRefQueryComparator`] to
/// allow for comparison of items with range bounds.
pub trait TypeRefQueryRangeComparator<'a, T, Q>: TypeRefQueryComparator<'a, T, Q>
where
  T: Type + ?Sized,
  Q: ?Sized,
{
  /// Returns `true` if `item` is contained in the range.
  #[inline]
  fn query_compare_contains<R>(&self, range: &R, item: &T::Ref<'a>) -> bool
  where
    R: ?Sized + RangeBounds<Q>,
  {
    let start = match range.start_bound() {
      Bound::Included(start) => self.query_compare_ref(item, start) != Ordering::Less,
      Bound::Excluded(start) => self.query_compare_ref(item, start) == Ordering::Greater,
      Bound::Unbounded => true,
    };

    let end = match range.end_bound() {
      Bound::Included(end) => self.query_compare_ref(item, end) != Ordering::Greater,
      Bound::Excluded(end) => self.query_compare_ref(item, end) == Ordering::Less,
      Bound::Unbounded => true,
    };

    start && end
  }
}

impl<'a, T, Q, C> TypeRefQueryRangeComparator<'a, T, Q> for C
where
  C: TypeRefQueryComparator<'a, T, Q>,
  T: Type + ?Sized,
  Q: ?Sized,
{
}

/// `QueryRangeComparator` is implemented as an extention to [`QueryComparator`] to
/// allow for comparison of items with range bounds.
pub trait QueryRangeComparator<T, Q>: QueryComparator<T, Q>
where
  T: ?Sized,
  Q: ?Sized,
{
  /// Returns `true` if `item` is contained in the range.
  #[inline]
  fn query_compare_contains<R>(&self, range: &R, item: &T) -> bool
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

impl<T, Q, C> QueryRangeComparator<T, Q> for C
where
  C: QueryComparator<T, Q>,
  Q: ?Sized,
  T: ?Sized,
{
}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  macro_rules! impl_traits {
    ($($ty:ty),+$(,)?) => {
      $(
        impl<T, C> Equivalentor<T> for $ty
        where
          C: Equivalentor<T>,
          T: ?Sized,
        {
          #[inline]
          fn equivalent(&self, a: &T, b: &T) -> bool
          {
            (**self).equivalent(a, b)
          }
        }

        impl<'a, T, C> TypeRefEquivalentor<'a, T> for $ty
        where
          C: TypeRefEquivalentor<'a, T>,
          T: Type + ?Sized,
        {
          #[inline]
          fn equivalent_ref(&self, a: &T, b: &T::Ref<'a>) -> bool {
            (**self).equivalent_ref(a, b)
          }

          /// Compare `a` to `b` and return `true` if they are equal.
          #[inline]
          fn equivalent_refs(
            &self,
            a: &T::Ref<'a>,
            b: &T::Ref<'a>,
          ) -> bool {
            (**self).equivalent_refs(a, b)
          }
        }

        impl<'a, T, Q, C> TypeRefQueryEquivalentor<'a, T, Q> for $ty
        where
          Q: ?Sized,
          T: Type + ?Sized,
          C: TypeRefQueryEquivalentor<'a, T, Q>,
        {
          #[inline]
          fn query_equivalent_ref(&self, a: &T::Ref<'a>, b: &Q) -> bool {
            (**self).query_equivalent_ref(a, b)
          }
        }

        impl<T, Q, C> QueryEquivalentor<T, Q> for $ty
        where
          Q: ?Sized,
          T: ?Sized,
          C: QueryEquivalentor<T, Q>,
        {
          #[inline]
          fn query_equivalent(&self, a: &T, b: &Q) -> bool {
            (**self).query_equivalent(a, b)
          }
        }

        impl<T, C> Comparator<T> for $ty
        where
          C: Comparator<T>,
          T: ?Sized,
        {
          #[inline]
          fn compare(&self, a: &T, b: &T) -> cmp::Ordering
          {
            (**self).compare(a, b)
          }
        }

        impl<'a, T, C> TypeRefComparator<'a, T> for $ty
        where
          C: TypeRefComparator<'a, T>,
          T: Type + ?Sized,
        {
          #[inline]
          fn compare_ref(&self, a: &T, b: &T::Ref<'a>) -> cmp::Ordering {
            (**self).compare_ref(a, b)
          }

          #[inline]
          fn compare_refs(
            &self,
            a: &T::Ref<'a>,
            b: &T::Ref<'a>,
          ) -> cmp::Ordering {
            (**self).compare_refs(a, b)
          }
        }

        impl<'a, T, Q, C> TypeRefQueryComparator<'a, T, Q> for $ty
        where
          Q: ?Sized,
          T: Type + ?Sized,
          C: TypeRefQueryComparator<'a, T, Q>,
        {
          #[inline]
          fn query_compare_ref(&self, a: &T::Ref<'a>, b: &Q) -> cmp::Ordering {
            (**self).query_compare_ref(a, b)
          }
        }

        impl<T, Q, C> QueryComparator<T, Q> for $ty
        where
          Q: ?Sized,
          T: ?Sized,
          C: QueryComparator<T, Q>,
        {
          #[inline]
          fn query_compare(&self, a: &T, b: &Q) -> cmp::Ordering {
            (**self).query_compare(a, b)
          }
        }
      )*
    };
  }

  impl_traits!(std::sync::Arc<C>, std::rc::Rc<C>);

  #[cfg(feature = "triomphe01")]
  impl_traits!(triomphe01::Arc<C>);
};
