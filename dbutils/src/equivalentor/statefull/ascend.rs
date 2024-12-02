use core::{cmp, marker::PhantomData};

use cheap_clone::CheapClone;
use equivalent::{Comparable, Equivalent};

use crate::types::Type;

use super::{
  Comparator, Equivalentor, QueryComparator, QueryEquivalentor, TypeRefComparator,
  TypeRefEquivalentor, TypeRefQueryComparator, TypeRefQueryEquivalentor,
};

/// Ascend is a comparator that compares items in ascending order.
pub struct Ascend<A: ?Sized>(PhantomData<A>);

impl<A: ?Sized> Ascend<A> {
  /// Create a new Ascend.
  #[inline]
  pub const fn new() -> Self {
    Self(PhantomData)
  }
}

impl<A: ?Sized> Default for Ascend<A> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<A: ?Sized> Clone for Ascend<A> {
  #[inline]
  fn clone(&self) -> Self {
    *self
  }
}

impl<A: ?Sized> CheapClone for Ascend<A> {}

impl<A: ?Sized> Copy for Ascend<A> {}

impl<A: ?Sized> PartialEq for Ascend<A> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.0.eq(&other.0)
  }
}

impl<A: ?Sized> core::fmt::Debug for Ascend<A> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    f.debug_struct("Ascend").finish()
  }
}

impl<A> Equivalentor for Ascend<A>
where
  A: Eq + ?Sized,
{
  type Type = A;

  #[inline]
  fn equivalent(&self, a: &Self::Type, b: &Self::Type) -> bool {
    a == b
  }
}

impl<'a, A> TypeRefEquivalentor<'a> for Ascend<A>
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

impl<A, Q> QueryEquivalentor<Q> for Ascend<A>
where
  A: Eq + ?Sized,
  Q: ?Sized + Equivalent<A>,
{
  #[inline]
  fn query_equivalent(&self, a: &Self::Type, b: &Q) -> bool {
    b.equivalent(a)
  }
}

impl<'a, A, Q> TypeRefQueryEquivalentor<'a, Q> for Ascend<A>
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

impl<A> Comparator for Ascend<A>
where
  A: Ord + ?Sized,
{
  #[inline]
  fn compare(&self, a: &Self::Type, b: &Self::Type) -> cmp::Ordering {
    a.cmp(b)
  }
}

impl<'a, A> TypeRefComparator<'a> for Ascend<A>
where
  A: ?Sized + Ord + Type,
  A::Ref<'a>: Comparable<A> + Ord,
{
  #[inline]
  fn compare_ref(&self, a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> cmp::Ordering {
    b.compare(a).reverse()
  }

  #[inline]
  fn compare_refs(
    &self,
    a: &<Self::Type as Type>::Ref<'a>,
    b: &<Self::Type as Type>::Ref<'a>,
  ) -> cmp::Ordering {
    a.cmp(b)
  }
}

impl<A, Q> QueryComparator<Q> for Ascend<A>
where
  A: ?Sized + Ord,
  Q: ?Sized + Comparable<A>,
{
  #[inline]
  fn query_compare(&self, a: &Self::Type, b: &Q) -> cmp::Ordering {
    b.compare(a).reverse()
  }
}

impl<'a, A, Q> TypeRefQueryComparator<'a, Q> for Ascend<A>
where
  A: ?Sized + Ord + Type,
  A::Ref<'a>: Comparable<A> + Ord,
  Q: ?Sized + Comparable<A::Ref<'a>>,
{
  #[inline]
  fn query_compare_ref(&self, a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> cmp::Ordering {
    b.compare(a).reverse()
  }
}
