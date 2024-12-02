use core::{cmp, marker::PhantomData};

use cheap_clone::CheapClone;

use crate::types::Type;

use super::{
  StaticComparator, StaticEquivalentor, StaticQueryComparator, StaticQueryEquivalentor,
  StaticTypeRefComparator, StaticTypeRefEquivalentor, StaticTypeRefQueryComparator,
  StaticTypeRefQueryEquivalentor,
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

impl<C> StaticEquivalentor for Reverse<C>
where
  C: StaticEquivalentor + ?Sized,
{
  type Type = C::Type;

  #[inline]
  fn equivalent(a: &Self::Type, b: &Self::Type) -> bool {
    C::equivalent(a, b)
  }
}

impl<'a, C> StaticTypeRefEquivalentor<'a> for Reverse<C>
where
  C: StaticTypeRefEquivalentor<'a> + ?Sized,
  C::Type: Type,
{
  #[inline]
  fn equivalent_ref(a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> bool {
    C::equivalent_ref(a, b)
  }

  #[inline]
  fn equivalent_refs(a: &<Self::Type as Type>::Ref<'a>, b: &<Self::Type as Type>::Ref<'a>) -> bool {
    C::equivalent_refs(a, b)
  }
}

impl<C, Q> StaticQueryEquivalentor<Q> for Reverse<C>
where
  C: StaticQueryEquivalentor<Q> + ?Sized,
{
  #[inline]
  fn query_equivalent(a: &Self::Type, b: &Q) -> bool {
    C::query_equivalent(a, b)
  }
}

impl<'a, C, Q> StaticTypeRefQueryEquivalentor<'a, Q> for Reverse<C>
where
  C: StaticTypeRefQueryEquivalentor<'a, Q> + ?Sized,
  C::Type: Type,
{
  #[inline]
  fn query_equivalent_ref(a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> bool {
    C::query_equivalent_ref(a, b)
  }
}

impl<C> StaticComparator for Reverse<C>
where
  C: StaticComparator + ?Sized,
{
  #[inline]
  fn compare(a: &Self::Type, b: &Self::Type) -> cmp::Ordering {
    C::compare(a, b).reverse()
  }
}

impl<'a, C> StaticTypeRefComparator<'a> for Reverse<C>
where
  C: StaticTypeRefComparator<'a> + ?Sized,
  C::Type: Type,
{
  #[inline]
  fn compare_ref(a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> cmp::Ordering {
    C::compare_ref(a, b).reverse()
  }

  #[inline]
  fn compare_refs(
    a: &<Self::Type as Type>::Ref<'a>,
    b: &<Self::Type as Type>::Ref<'a>,
  ) -> cmp::Ordering {
    C::compare_refs(a, b).reverse()
  }
}

impl<C, Q> StaticQueryComparator<Q> for Reverse<C>
where
  C: StaticQueryComparator<Q> + ?Sized,
{
  #[inline]
  fn query_compare(a: &Self::Type, b: &Q) -> cmp::Ordering {
    C::query_compare(a, b).reverse()
  }
}

impl<'a, C, Q> StaticTypeRefQueryComparator<'a, Q> for Reverse<C>
where
  C: StaticTypeRefQueryComparator<'a, Q> + ?Sized,
  C::Type: Type,
{
  #[inline]
  fn query_compare_ref(a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> cmp::Ordering {
    C::query_compare_ref(a, b).reverse()
  }
}
