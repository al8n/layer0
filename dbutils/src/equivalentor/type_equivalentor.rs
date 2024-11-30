use core::{
  cmp::{self, Ordering},
  ops::{Bound, RangeBounds},
};

use crate::types::Type;

/// Custom equivalence trait.
pub trait TypeEquivalentor {
  /// The base type for comparison.
  type Type: ?Sized + Type;

  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent(&self, a: &Self::Type, b: &Self::Type) -> bool;
}

/// Custom equivalence trait.
pub trait TypeRefEquivalentor<'a>: TypeEquivalentor {
  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent_ref(&self, a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> bool;

  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent_refs(
    &self,
    a: &<Self::Type as Type>::Ref<'a>,
    b: &<Self::Type as Type>::Ref<'a>,
  ) -> bool;
}

/// Custom equivalence trait for query purpose.
pub trait TypeRefQueryEquivalentor<'a, Q: ?Sized>: TypeRefEquivalentor<'a> {
  /// Compare `a` to `b` and return `true` if they are equal.
  fn query_equivalent_ref(&self, a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> bool;
}

// impl<'a, C> TypeRefQueryEquivalentor<'a, <C::Type as Type>::Ref<'a>> for C
// where
//   C: TypeRefEquivalentor<'a>,
// {
//   #[inline]
//   fn query_equivalent_ref(&self, a: &<C::Type as Type>::Ref<'a>, b: &<C::Type as Type>::Ref<'a>) -> bool {
//     self.equivalent_refs(a, b)
//   }
// }

/// Custom equivalence trait for query purpose.
pub trait TypeQueryEquivalentor<Q: ?Sized>: TypeEquivalentor {
  /// Compare `a` to `b` and return `true` if they are equal.
  fn query_equivalent(&self, a: &Self::Type, b: &Q) -> bool;
}

// impl<C> TypeQueryEquivalentor<C::Type> for C
// where
//   C: TypeEquivalentor,
// {
//   #[inline]
//   fn query_equivalent(&self, a: &Self::Type, b: &C::Type) -> bool {
//     self.equivalent(a, b)
//   }
// }

/// Custom ordering trait.
pub trait TypeComparator: TypeEquivalentor {
  /// Compare `a` to `b` and return their ordering.
  fn compare(&self, a: &Self::Type, b: &Self::Type) -> cmp::Ordering;
}

/// Custom ordering trait.
pub trait TypeRefComparator<'a>: TypeComparator + TypeRefEquivalentor<'a> {
  /// Compare `a` to `b` and return their ordering.
  fn compare_ref(&self, a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> cmp::Ordering;

  /// Compare `a` to `b` and return their ordering.
  fn compare_refs(
    &self,
    a: &<Self::Type as Type>::Ref<'a>,
    b: &<Self::Type as Type>::Ref<'a>,
  ) -> cmp::Ordering;
}

/// Custom ordering trait for querying purpose.
pub trait TypeRefQueryComparator<'a, Q: ?Sized>:
  TypeRefComparator<'a> + TypeRefQueryEquivalentor<'a, Q>
{
  /// Compare `a` to `b` and return their ordering.
  fn query_compare_ref(&self, a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> cmp::Ordering;
}

/// Custom ordering trait for querying purpose.
pub trait TypeQueryComparator<Q: ?Sized>: TypeComparator + TypeQueryEquivalentor<Q> {
  /// Compare `a` to `b` and return their ordering.
  fn query_compare(&self, a: &Self::Type, b: &Q) -> cmp::Ordering;
}

/// `TypeRefQueryRangeComparator` is implemented as an extention to [`TypeRefQueryComparator`] to
/// allow for comparison of items with range bounds.
pub trait TypeRefQueryRangeComparator<'a, Q: ?Sized>: TypeRefQueryComparator<'a, Q> {
  /// Returns `true` if `item` is contained in the range.
  #[inline]
  fn query_compare_contains<R>(&self, range: &R, item: &<Self::Type as Type>::Ref<'a>) -> bool
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

impl<'a, Q: ?Sized, C> TypeRefQueryRangeComparator<'a, Q> for C where
  C: TypeRefQueryComparator<'a, Q>
{
}

/// `TypeQueryRangeComparator` is implemented as an extention to [`TypeQueryComparator`] to
/// allow for comparison of items with range bounds.
pub trait TypeQueryRangeComparator<Q: ?Sized>: TypeQueryComparator<Q> {
  /// Returns `true` if `item` is contained in the range.
  #[inline]
  fn query_compare_contains<R>(&self, range: &R, item: &Self::Type) -> bool
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

impl<Q: ?Sized, C> TypeQueryRangeComparator<Q> for C where C: TypeQueryComparator<Q> {}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  macro_rules! impl_traits {
    ($($ty:ty),+$(,)?) => {
      $(
        impl<C> TypeEquivalentor for $ty
        where
          C: TypeEquivalentor,
        {
          type Type = C::Type;

          #[inline]
          fn equivalent(&self, a: &Self::Type, b: &Self::Type) -> bool
          {
            (**self).equivalent(a, b)
          }
        }

        impl<'a, C> TypeRefEquivalentor<'a> for $ty
        where
          C: TypeRefEquivalentor<'a>,
        {
          #[inline]
          fn equivalent_ref(&self, a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> bool {
            (**self).equivalent_ref(a, b)
          }

          /// Compare `a` to `b` and return `true` if they are equal.
          #[inline]
          fn equivalent_refs(
            &self,
            a: &<Self::Type as Type>::Ref<'a>,
            b: &<Self::Type as Type>::Ref<'a>,
          ) -> bool {
            (**self).equivalent_refs(a, b)
          }
        }

        impl<'a, Q, C> TypeRefQueryEquivalentor<'a, Q> for $ty
        where
          Q: ?Sized,
          C: TypeRefQueryEquivalentor<'a, Q>,
        {
          #[inline]
          fn query_equivalent_ref(&self, a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> bool {
            (**self).query_equivalent_ref(a, b)
          }
        }

        impl<Q, C> TypeQueryEquivalentor<Q> for $ty
        where
          Q: ?Sized,
          C: TypeQueryEquivalentor<Q>,
        {
          #[inline]
          fn query_equivalent(&self, a: &Self::Type, b: &Q) -> bool {
            (**self).query_equivalent(a, b)
          }
        }

        impl<C> TypeComparator for $ty
        where
          C: TypeComparator,
        {
          #[inline]
          fn compare(&self, a: &Self::Type, b: &Self::Type) -> cmp::Ordering
          {
            (**self).compare(a, b)
          }
        }

        impl<'a, C> TypeRefComparator<'a> for $ty
        where
          C: TypeRefComparator<'a>,
        {
          #[inline]
          fn compare_ref(&self, a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> cmp::Ordering {
            (**self).compare_ref(a, b)
          }

          #[inline]
          fn compare_refs(
            &self,
            a: &<Self::Type as Type>::Ref<'a>,
            b: &<Self::Type as Type>::Ref<'a>,
          ) -> cmp::Ordering {
            (**self).compare_refs(a, b)
          }
        }

        impl<'a, Q, C> TypeRefQueryComparator<'a, Q> for $ty
        where
          Q: ?Sized,
          C: TypeRefQueryComparator<'a, Q>,
        {
          #[inline]
          fn query_compare_ref(&self, a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> cmp::Ordering {
            (**self).query_compare_ref(a, b)
          }
        }

        impl<Q, C> TypeQueryComparator<Q> for $ty
        where
          Q: ?Sized,
          C: TypeQueryComparator<Q>,
        {
          #[inline]
          fn query_compare(&self, a: &Self::Type, b: &Q) -> cmp::Ordering {
            (**self).query_compare(a, b)
          }
        }
      )*
    };
  }

  impl_traits!(std::sync::Arc<C>, std::rc::Rc<C>, std::boxed::Box<C>);

  #[cfg(feature = "triomphe01")]
  impl_traits!(triomphe01::Arc<C>);
};
