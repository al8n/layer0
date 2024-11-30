use core::{cmp, marker::PhantomData};

use cheap_clone::CheapClone;
use equivalent::{Comparable, Equivalent};

use crate::types::Type;

use super::{
  Comparator, Equivalentor, QueryComparator, QueryEquivalentor, StaticComparator,
  StaticEquivalentor, StaticQueryComparator, StaticQueryEquivalentor, StaticTypeRefComparator,
  StaticTypeRefEquivalentor, StaticTypeRefQueryComparator, StaticTypeRefQueryEquivalentor,
  TypeRefComparator, TypeRefEquivalentor, TypeRefQueryComparator, TypeRefQueryEquivalentor,
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

impl<A: ?Sized> PartialEq for Descend<A> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.0.eq(&other.0)
  }
}

impl<A> StaticEquivalentor for Descend<A>
where
  A: Eq + ?Sized,
{
  type Type = A;

  #[inline]
  fn equivalent(a: &Self::Type, b: &Self::Type) -> bool {
    a == b
  }
}

impl<'a, A> StaticTypeRefEquivalentor<'a> for Descend<A>
where
  A: ?Sized + Eq + Type,
  A::Ref<'a>: Equivalent<A> + Eq,
{
  #[inline]
  fn equivalent_ref(a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> bool {
    b.equivalent(a)
  }

  #[inline]
  fn equivalent_refs(a: &<Self::Type as Type>::Ref<'a>, b: &<Self::Type as Type>::Ref<'a>) -> bool {
    a == b
  }
}

impl<A, Q> StaticQueryEquivalentor<Q> for Descend<A>
where
  A: Eq + ?Sized,
  Q: ?Sized + Equivalent<A>,
{
  #[inline]
  fn query_equivalent(a: &Self::Type, b: &Q) -> bool {
    b.equivalent(a)
  }
}

impl<'a, A, Q> StaticTypeRefQueryEquivalentor<'a, Q> for Descend<A>
where
  A: ?Sized + Eq + Type,
  A::Ref<'a>: Equivalent<A> + Eq,
  Q: ?Sized + Equivalent<A::Ref<'a>>,
{
  #[inline]
  fn query_equivalent_ref(a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> bool {
    b.equivalent(a)
  }
}

impl<A> StaticComparator for Descend<A>
where
  A: Ord + ?Sized,
{
  #[inline]
  fn compare(a: &Self::Type, b: &Self::Type) -> cmp::Ordering {
    a.cmp(b).reverse()
  }
}

impl<'a, A> StaticTypeRefComparator<'a> for Descend<A>
where
  A: ?Sized + Ord + Type,
  A::Ref<'a>: Comparable<A> + Ord,
{
  #[inline]
  fn compare_ref(a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> cmp::Ordering {
    b.compare(a)
  }

  #[inline]
  fn compare_refs(
    a: &<Self::Type as Type>::Ref<'a>,
    b: &<Self::Type as Type>::Ref<'a>,
  ) -> cmp::Ordering {
    b.cmp(a)
  }
}

impl<A, Q> StaticQueryComparator<Q> for Descend<A>
where
  A: ?Sized + Ord,
  Q: ?Sized + Comparable<A>,
{
  #[inline]
  fn query_compare(a: &Self::Type, b: &Q) -> cmp::Ordering {
    b.compare(a)
  }
}

impl<'a, A, Q> StaticTypeRefQueryComparator<'a, Q> for Descend<A>
where
  A: ?Sized + Ord + Type,
  A::Ref<'a>: Comparable<A> + Ord,
  Q: ?Sized + Comparable<A::Ref<'a>>,
{
  #[inline]
  fn query_compare_ref(a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> cmp::Ordering {
    b.compare(a)
  }
}

impl<A> Equivalentor for Descend<A>
where
  Descend<A>: StaticEquivalentor,
  A: ?Sized,
{
  type Type = <Descend<A> as StaticEquivalentor>::Type;

  #[inline]
  fn equivalent(&self, a: &Self::Type, b: &Self::Type) -> bool {
    <Descend<A> as StaticEquivalentor>::equivalent(a, b)
  }
}

impl<Q, A> QueryEquivalentor<Q> for Descend<A>
where
  Q: ?Sized,
  A: ?Sized,
  Descend<A>: StaticQueryEquivalentor<Q>,
{
  #[inline]
  fn query_equivalent(&self, a: &Self::Type, b: &Q) -> bool {
    <Descend<A> as StaticQueryEquivalentor<Q>>::query_equivalent(a, b)
  }
}

impl<'a, A> TypeRefEquivalentor<'a> for Descend<A>
where
  Descend<A>: StaticTypeRefEquivalentor<'a>,
  Self::Type: Type,
  A: ?Sized,
{
  #[inline]
  fn equivalent_ref(&self, a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> bool {
    <Descend<A> as StaticTypeRefEquivalentor<'a>>::equivalent_ref(a, b)
  }

  #[inline]
  fn equivalent_refs(
    &self,
    a: &<Self::Type as Type>::Ref<'a>,
    b: &<Self::Type as Type>::Ref<'a>,
  ) -> bool {
    <Descend<A> as StaticTypeRefEquivalentor<'a>>::equivalent_refs(a, b)
  }
}

impl<'a, Q, A> TypeRefQueryEquivalentor<'a, Q> for Descend<A>
where
  Q: ?Sized,
  Descend<A>: StaticTypeRefQueryEquivalentor<'a, Q>,
  A: ?Sized,
  Self::Type: Type,
{
  #[inline]
  fn query_equivalent_ref(&self, a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> bool {
    <Descend<A> as StaticTypeRefQueryEquivalentor<'a, Q>>::query_equivalent_ref(a, b)
  }
}

impl<A> Comparator for Descend<A>
where
  Descend<A>: StaticComparator,
  A: ?Sized,
{
  #[inline]
  fn compare(&self, a: &Self::Type, b: &Self::Type) -> cmp::Ordering {
    <Descend<A> as StaticComparator>::compare(a, b)
  }
}

impl<Q, A> QueryComparator<Q> for Descend<A>
where
  Q: ?Sized,
  Descend<A>: StaticQueryComparator<Q>,
  A: ?Sized,
{
  #[inline]
  fn query_compare(&self, a: &Self::Type, b: &Q) -> cmp::Ordering {
    <Descend<A> as StaticQueryComparator<Q>>::query_compare(a, b)
  }
}

impl<'a, A> TypeRefComparator<'a> for Descend<A>
where
  Descend<A>: StaticTypeRefComparator<'a>,
  Self::Type: Type,
  A: ?Sized,
{
  #[inline]
  fn compare_ref(&self, a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> cmp::Ordering {
    <Descend<A> as StaticTypeRefComparator<'a>>::compare_ref(a, b)
  }

  #[inline]
  fn compare_refs(
    &self,
    a: &<Self::Type as Type>::Ref<'a>,
    b: &<Self::Type as Type>::Ref<'a>,
  ) -> cmp::Ordering {
    <Descend<A> as StaticTypeRefComparator<'a>>::compare_refs(a, b)
  }
}

impl<'a, Q, A> TypeRefQueryComparator<'a, Q> for Descend<A>
where
  Q: ?Sized,
  Descend<A>: StaticTypeRefQueryComparator<'a, Q>,
  A: ?Sized,
  Self::Type: Type,
{
  #[inline]
  fn query_compare_ref(&self, a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> cmp::Ordering {
    <Descend<A> as StaticTypeRefQueryComparator<'a, Q>>::query_compare_ref(a, b)
  }
}
