use core::{cmp, marker::PhantomData};

use cheap_clone::CheapClone;
use equivalent::{Comparable, Equivalent};

use crate::types::Type;

use super::{
  Comparator, Equivalentor, QueryComparator, QueryEquivalentor, TypeRefComparator,
  TypeRefEquivalentor, TypeRefQueryComparator, TypeRefQueryEquivalentor,
};

/// Descend is a comparator that compares byte slices in ascending order.
pub struct Descend<A: ?Sized>(PhantomData<A>);

impl<A: ?Sized> Descend<A> {
  /// Create a new Descend.
  #[inline]
  pub const fn new() -> Self {
    Self(PhantomData)
  }
}

impl<A: ?Sized> Default for Descend<A> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<A: ?Sized> Clone for Descend<A> {
  #[inline]
  fn clone(&self) -> Self {
    *self
  }
}

impl<A: ?Sized> CheapClone for Descend<A> {}

impl<A: ?Sized> Copy for Descend<A> {}

impl<A: ?Sized> core::fmt::Debug for Descend<A> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    f.debug_struct("Descend").finish()
  }
}

impl<A> Equivalentor for Descend<A>
where
  A: Eq + ?Sized,
{
  type Type = A;

  #[inline]
  fn equivalent(&self, a: &Self::Type, b: &Self::Type) -> bool {
    a == b
  }
}

impl<'a, A> TypeRefEquivalentor<'a> for Descend<A>
where
  A: ?Sized + Eq + Type,
  A::Ref<'a>: Equivalent<A> + Eq,
{
  #[inline]
  fn equivalent_ref(&self, a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> bool {
    b.equivalent(a)
  }

  #[inline]
  fn equivalent_refs(
    &self,
    a: &<Self::Type as Type>::Ref<'a>,
    b: &<Self::Type as Type>::Ref<'a>,
  ) -> bool {
    a == b
  }
}

impl<A, Q> QueryEquivalentor<Q> for Descend<A>
where
  A: Eq + ?Sized,
  Q: ?Sized + Equivalent<A>,
{
  #[inline]
  fn query_equivalent(&self, a: &Self::Type, b: &Q) -> bool {
    b.equivalent(a)
  }
}

impl<'a, A, Q> TypeRefQueryEquivalentor<'a, Q> for Descend<A>
where
  A: ?Sized + Eq + Type,
  A::Ref<'a>: Equivalent<A> + Eq,
  Q: ?Sized + Equivalent<A::Ref<'a>>,
{
  #[inline]
  fn query_equivalent_ref(&self, a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> bool {
    b.equivalent(a)
  }
}

impl<A> Comparator for Descend<A>
where
  A: Ord + ?Sized,
{
  #[inline]
  fn compare(&self, a: &Self::Type, b: &Self::Type) -> cmp::Ordering {
    a.cmp(b).reverse()
  }
}

impl<'a, A> TypeRefComparator<'a> for Descend<A>
where
  A: ?Sized + Ord + Type,
  A::Ref<'a>: Comparable<A> + Ord,
{
  #[inline]
  fn compare_ref(&self, a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> cmp::Ordering {
    b.compare(a)
  }

  #[inline]
  fn compare_refs(
    &self,
    a: &<Self::Type as Type>::Ref<'a>,
    b: &<Self::Type as Type>::Ref<'a>,
  ) -> cmp::Ordering {
    b.cmp(a)
  }
}

impl<A, Q> QueryComparator<Q> for Descend<A>
where
  A: ?Sized + Ord,
  Q: ?Sized + Comparable<A>,
{
  #[inline]
  fn query_compare(&self, a: &Self::Type, b: &Q) -> cmp::Ordering {
    b.compare(a)
  }
}

impl<'a, A, Q> TypeRefQueryComparator<'a, Q> for Descend<A>
where
  A: ?Sized + Ord + Type,
  A::Ref<'a>: Comparable<A> + Ord,
  Q: ?Sized + Comparable<A::Ref<'a>>,
{
  #[inline]
  fn query_compare_ref(&self, a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> cmp::Ordering {
    b.compare(a)
  }
}
