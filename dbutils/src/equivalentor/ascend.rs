use core::cmp;

use cheap_clone::CheapClone;

use crate::{
  equivalent::{Comparable, Equivalent},
  types::Type,
};

use super::{
  Comparator, Equivalentor, QueryComparator, QueryEquivalentor, StaticComparator,
  StaticEquivalentor, StaticQueryComparator, StaticQueryEquivalentor, StaticTypeRefComparator,
  StaticTypeRefEquivalentor, StaticTypeRefQueryComparator, StaticTypeRefQueryEquivalentor,
  TypeRefComparator, TypeRefEquivalentor, TypeRefQueryComparator, TypeRefQueryEquivalentor,
};

/// Ascend is a comparator that compares items in ascending order.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ascend;

impl Ascend {
  /// Create a new Ascend.
  #[inline]
  pub const fn new() -> Self {
    Self
  }
}

impl CheapClone for Ascend {}

impl<A> StaticEquivalentor<A> for Ascend
where
  A: Eq + ?Sized,
{
  #[inline]
  fn equivalent(a: &A, b: &A) -> bool {
    a == b
  }
}

impl<'a, A> StaticTypeRefEquivalentor<'a, A> for Ascend
where
  A: ?Sized + Eq + Type,
  A::Ref<'a>: Equivalent<A> + Eq,
{
  #[inline]
  fn equivalent_ref(a: &A, b: &<A as Type>::Ref<'a>) -> bool {
    b.equivalent(a)
  }

  #[inline]
  fn equivalent_refs(a: &<A as Type>::Ref<'a>, b: &<A as Type>::Ref<'a>) -> bool {
    a == b
  }
}

impl<A, Q> StaticQueryEquivalentor<A, Q> for Ascend
where
  A: Eq + ?Sized,
  Q: ?Sized + Equivalent<A>,
{
  #[inline]
  fn query_equivalent(a: &A, b: &Q) -> bool {
    b.equivalent(a)
  }
}

impl<'a, A, Q> StaticTypeRefQueryEquivalentor<'a, A, Q> for Ascend
where
  A: ?Sized + Eq + Type,
  A::Ref<'a>: Equivalent<A> + Eq,
  Q: ?Sized + Equivalent<A::Ref<'a>>,
{
  #[inline]
  fn query_equivalent_ref(a: &<A as Type>::Ref<'a>, b: &Q) -> bool {
    b.equivalent(a)
  }
}

impl<A> StaticComparator<A> for Ascend
where
  A: Ord + ?Sized,
{
  #[inline]
  fn compare(a: &A, b: &A) -> cmp::Ordering {
    a.cmp(b)
  }
}

impl<'a, A> StaticTypeRefComparator<'a, A> for Ascend
where
  A: ?Sized + Ord + Type,
  A::Ref<'a>: Comparable<A> + Ord,
{
  #[inline]
  fn compare_ref(a: &A, b: &<A as Type>::Ref<'a>) -> cmp::Ordering {
    b.compare(a).reverse()
  }

  #[inline]
  fn compare_refs(a: &<A as Type>::Ref<'a>, b: &<A as Type>::Ref<'a>) -> cmp::Ordering {
    a.cmp(b)
  }
}

impl<A, Q> StaticQueryComparator<A, Q> for Ascend
where
  A: ?Sized + Ord,
  Q: ?Sized + Comparable<A>,
{
  #[inline]
  fn query_compare(a: &A, b: &Q) -> cmp::Ordering {
    b.compare(a).reverse()
  }
}

impl<'a, A, Q> StaticTypeRefQueryComparator<'a, A, Q> for Ascend
where
  A: ?Sized + Ord + Type,
  A::Ref<'a>: Comparable<A> + Ord,
  Q: ?Sized + Comparable<A::Ref<'a>>,
{
  #[inline]
  fn query_compare_ref(a: &<A as Type>::Ref<'a>, b: &Q) -> cmp::Ordering {
    b.compare(a).reverse()
  }
}

impl<A> Equivalentor<A> for Ascend
where
  Ascend: StaticEquivalentor<A>,
  A: ?Sized,
{
  #[inline]
  fn equivalent(&self, a: &A, b: &A) -> bool {
    <Ascend as StaticEquivalentor<_>>::equivalent(a, b)
  }
}

impl<A, Q> QueryEquivalentor<A, Q> for Ascend
where
  Q: ?Sized,
  A: ?Sized,
  Ascend: StaticQueryEquivalentor<A, Q>,
{
  #[inline]
  fn query_equivalent(&self, a: &A, b: &Q) -> bool {
    <Ascend as StaticQueryEquivalentor<A, Q>>::query_equivalent(a, b)
  }
}

impl<'a, A> TypeRefEquivalentor<'a, A> for Ascend
where
  Ascend: StaticTypeRefEquivalentor<'a, A>,
  A: Type + ?Sized,
{
  #[inline]
  fn equivalent_ref(&self, a: &A, b: &<A as Type>::Ref<'a>) -> bool {
    <Ascend as StaticTypeRefEquivalentor<'a, A>>::equivalent_ref(a, b)
  }

  #[inline]
  fn equivalent_refs(&self, a: &<A as Type>::Ref<'a>, b: &<A as Type>::Ref<'a>) -> bool {
    <Ascend as StaticTypeRefEquivalentor<'a, A>>::equivalent_refs(a, b)
  }
}

impl<'a, A, Q> TypeRefQueryEquivalentor<'a, A, Q> for Ascend
where
  Q: ?Sized,
  Ascend: StaticTypeRefQueryEquivalentor<'a, A, Q>,
  A: Type + ?Sized,
{
  #[inline]
  fn query_equivalent_ref(&self, a: &<A as Type>::Ref<'a>, b: &Q) -> bool {
    <Ascend as StaticTypeRefQueryEquivalentor<'a, A, Q>>::query_equivalent_ref(a, b)
  }
}

impl<A> Comparator<A> for Ascend
where
  Ascend: StaticComparator<A>,
  A: ?Sized,
{
  #[inline]
  fn compare(&self, a: &A, b: &A) -> cmp::Ordering {
    <Ascend as StaticComparator<A>>::compare(a, b)
  }
}

impl<Q, A> QueryComparator<A, Q> for Ascend
where
  Q: ?Sized,
  Ascend: StaticQueryComparator<A, Q>,
  A: ?Sized,
{
  #[inline]
  fn query_compare(&self, a: &A, b: &Q) -> cmp::Ordering {
    <Ascend as StaticQueryComparator<A, Q>>::query_compare(a, b)
  }
}

impl<'a, A> TypeRefComparator<'a, A> for Ascend
where
  A: Type,
  A: ?Sized,
  Ascend: StaticTypeRefComparator<'a, A>,
{
  #[inline]
  fn compare_ref(&self, a: &A, b: &<A as Type>::Ref<'a>) -> cmp::Ordering {
    <Ascend as StaticTypeRefComparator<'a, A>>::compare_ref(a, b)
  }

  #[inline]
  fn compare_refs(&self, a: &<A as Type>::Ref<'a>, b: &<A as Type>::Ref<'a>) -> cmp::Ordering {
    <Ascend as StaticTypeRefComparator<'a, A>>::compare_refs(a, b)
  }
}

impl<'a, Q, A> TypeRefQueryComparator<'a, A, Q> for Ascend
where
  Q: ?Sized,
  Ascend: StaticTypeRefQueryComparator<'a, A, Q>,
  A: Type + ?Sized,
{
  #[inline]
  fn query_compare_ref(&self, a: &<A as Type>::Ref<'a>, b: &Q) -> cmp::Ordering {
    <Ascend as StaticTypeRefQueryComparator<'a, A, Q>>::query_compare_ref(a, b)
  }
}
