use core::{cmp, marker::PhantomData};

use cheap_clone::CheapClone;

use crate::types::Type;

use super::{
  Comparator, Equivalentor, QueryComparator, QueryEquivalentor, StaticComparator,
  StaticEquivalentor, StaticQueryComparator, StaticQueryEquivalentor, StaticTypeRefComparator,
  StaticTypeRefEquivalentor, StaticTypeRefQueryComparator, StaticTypeRefQueryEquivalentor,
  TypeRefComparator, TypeRefEquivalentor, TypeRefQueryComparator, TypeRefQueryEquivalentor,
};

/// Reverse is a comparator that compares byte slices in ascending order.
pub struct Reverse<C: ?Sized>(PhantomData<C>);

impl<C: ?Sized> Reverse<C> {
  /// Create a new Reverse.
  #[inline]
  pub const fn new() -> Self {
    Self(PhantomData)
  }
}

impl<C: ?Sized> Default for Reverse<C> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<C: ?Sized> Clone for Reverse<C> {
  #[inline]
  fn clone(&self) -> Self {
    *self
  }
}

impl<C: ?Sized> CheapClone for Reverse<C> {}

impl<C: ?Sized> Copy for Reverse<C> {}

impl<C: ?Sized> core::fmt::Debug for Reverse<C> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    f.debug_struct("Reverse").finish()
  }
}

impl<C: ?Sized> PartialEq for Reverse<C> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.0.eq(&other.0)
  }
}

impl<T, C> StaticEquivalentor<T> for Reverse<C>
where
  C: StaticEquivalentor<T> + ?Sized,
  T: ?Sized,
{
  #[inline]
  fn equivalent(a: &T, b: &T) -> bool {
    C::equivalent(a, b)
  }
}

impl<T, C> StaticTypeRefEquivalentor<T> for Reverse<C>
where
  C: StaticTypeRefEquivalentor<T> + ?Sized,
  T: Type + ?Sized,
{
  #[inline]
  fn equivalent_ref(a: &T, b: &T::Ref<'_>) -> bool {
    C::equivalent_ref(a, b)
  }

  #[inline]
  fn equivalent_refs<'a>(a: &T::Ref<'a>, b: &T::Ref<'a>) -> bool {
    C::equivalent_refs(a, b)
  }
}

impl<C, T, Q> StaticQueryEquivalentor<T, Q> for Reverse<C>
where
  C: StaticQueryEquivalentor<T, Q> + ?Sized,
  T: ?Sized,
  Q: ?Sized,
{
  #[inline]
  fn query_equivalent(a: &T, b: &Q) -> bool {
    C::query_equivalent(a, b)
  }
}

impl<C, T, Q> StaticTypeRefQueryEquivalentor<T, Q> for Reverse<C>
where
  C: StaticTypeRefQueryEquivalentor<T, Q> + ?Sized,
  T: Type + ?Sized,
  Q: ?Sized,
{
  #[inline]
  fn query_equivalent_ref(a: &T::Ref<'_>, b: &Q) -> bool {
    C::query_equivalent_ref(a, b)
  }
}

impl<C, T> StaticComparator<T> for Reverse<C>
where
  C: StaticComparator<T> + ?Sized,
  T: ?Sized,
{
  #[inline]
  fn compare(a: &T, b: &T) -> cmp::Ordering {
    C::compare(a, b).reverse()
  }
}

impl<C, T> StaticTypeRefComparator<T> for Reverse<C>
where
  C: StaticTypeRefComparator<T> + ?Sized,
  T: Type + ?Sized,
{
  #[inline]
  fn compare_ref(a: &T, b: &T::Ref<'_>) -> cmp::Ordering {
    C::compare_ref(a, b).reverse()
  }

  #[inline]
  fn compare_refs<'a>(a: &T::Ref<'a>, b: &T::Ref<'a>) -> cmp::Ordering {
    C::compare_refs(a, b).reverse()
  }
}

impl<C, T, Q> StaticQueryComparator<T, Q> for Reverse<C>
where
  C: StaticQueryComparator<T, Q> + ?Sized,
  T: ?Sized,
  Q: ?Sized,
{
  #[inline]
  fn query_compare(a: &T, b: &Q) -> cmp::Ordering {
    C::query_compare(a, b).reverse()
  }
}

impl<C, T, Q> StaticTypeRefQueryComparator<T, Q> for Reverse<C>
where
  C: StaticTypeRefQueryComparator<T, Q> + ?Sized,
  T: Type + ?Sized,
  Q: ?Sized,
{
  #[inline]
  fn query_compare_ref(a: &T::Ref<'_>, b: &Q) -> cmp::Ordering {
    C::query_compare_ref(a, b).reverse()
  }
}

impl<C, T> Equivalentor<T> for Reverse<C>
where
  C: StaticEquivalentor<T> + ?Sized,
  T: ?Sized,
{
  #[inline]
  fn equivalent(&self, a: &T, b: &T) -> bool {
    C::equivalent(a, b)
  }
}

impl<C, T> TypeRefEquivalentor<T> for Reverse<C>
where
  C: StaticTypeRefEquivalentor<T> + ?Sized,
  T: Type + ?Sized,
{
  #[inline]
  fn equivalent_ref(&self, a: &T, b: &T::Ref<'_>) -> bool {
    C::equivalent_ref(a, b)
  }

  #[inline]
  fn equivalent_refs<'a>(&self, a: &T::Ref<'a>, b: &T::Ref<'a>) -> bool {
    C::equivalent_refs(a, b)
  }
}

impl<C, Q, T> QueryEquivalentor<T, Q> for Reverse<C>
where
  C: StaticQueryEquivalentor<T, Q> + ?Sized,
  T: ?Sized,
  Q: ?Sized,
{
  #[inline]
  fn query_equivalent(&self, a: &T, b: &Q) -> bool {
    C::query_equivalent(a, b)
  }
}

impl<C, T, Q> TypeRefQueryEquivalentor<T, Q> for Reverse<C>
where
  C: StaticTypeRefQueryEquivalentor<T, Q> + ?Sized,
  T: Type + ?Sized,
  Q: ?Sized,
{
  #[inline]
  fn query_equivalent_ref(&self, a: &T::Ref<'_>, b: &Q) -> bool {
    C::query_equivalent_ref(a, b)
  }
}

impl<C, T> Comparator<T> for Reverse<C>
where
  C: StaticComparator<T> + ?Sized,
  T: ?Sized,
{
  #[inline]
  fn compare(&self, a: &T, b: &T) -> cmp::Ordering {
    C::compare(a, b).reverse()
  }
}

impl<C, T> TypeRefComparator<T> for Reverse<C>
where
  C: StaticTypeRefComparator<T> + ?Sized,
  T: Type + ?Sized,
{
  #[inline]
  fn compare_ref(&self, a: &T, b: &T::Ref<'_>) -> cmp::Ordering {
    C::compare_ref(a, b).reverse()
  }

  #[inline]
  fn compare_refs<'a>(&self, a: &T::Ref<'a>, b: &T::Ref<'a>) -> cmp::Ordering {
    C::compare_refs(a, b).reverse()
  }
}

impl<C, T, Q> QueryComparator<T, Q> for Reverse<C>
where
  C: StaticQueryComparator<T, Q> + ?Sized,
  T: ?Sized,
  Q: ?Sized,
{
  #[inline]
  fn query_compare(&self, a: &T, b: &Q) -> cmp::Ordering {
    C::query_compare(a, b).reverse()
  }
}

impl<C, T, Q> TypeRefQueryComparator<T, Q> for Reverse<C>
where
  C: StaticTypeRefQueryComparator<T, Q> + ?Sized,
  T: Type + ?Sized,
  Q: ?Sized,
{
  #[inline]
  fn query_compare_ref(&self, a: &T::Ref<'_>, b: &Q) -> cmp::Ordering {
    C::query_compare_ref(a, b).reverse()
  }
}
