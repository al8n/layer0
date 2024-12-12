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

/// Descend is a comparator that compares byte slices in ascending order.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Descend;

impl Descend {
  /// Create a new Descend.
  #[inline]
  pub const fn new() -> Self {
    Self
  }
}

impl CheapClone for Descend {}

impl<A> StaticEquivalentor<A> for Descend
where
  A: Eq + ?Sized,
{
  #[inline]
  fn equivalent(a: &A, b: &A) -> bool {
    a == b
  }
}

impl<A> StaticTypeRefEquivalentor<A> for Descend
where
  A: ?Sized + Eq + Type + for<'a> Equivalent<A::Ref<'a>>,
  for<'a> A::Ref<'a>: Eq,
{
  #[inline]
  fn equivalent_ref(a: &A, b: &A::Ref<'_>) -> bool {
    a.equivalent(b)
  }

  #[inline]
  fn equivalent_refs<'a>(a: &A::Ref<'a>, b: &A::Ref<'a>) -> bool {
    a == b
  }
}

impl<A, Q> StaticQueryEquivalentor<A, Q> for Descend
where
  A: Eq + ?Sized + Equivalent<Q>,
  Q: ?Sized,
{
  #[inline]
  fn query_equivalent(a: &A, b: &Q) -> bool {
    a.equivalent(b)
  }
}

impl<A, Q> StaticTypeRefQueryEquivalentor<A, Q> for Descend
where
  A: ?Sized + Eq + Type + for<'a> Equivalent<A::Ref<'a>>,
  for<'a> A::Ref<'a>: Eq + Equivalent<Q>,
  Q: ?Sized,
{
  #[inline]
  fn query_equivalent_ref(a: &A::Ref<'_>, b: &Q) -> bool {
    a.equivalent(b)
  }
}

impl<A> StaticComparator<A> for Descend
where
  A: Ord + ?Sized,
{
  #[inline]
  fn compare(a: &A, b: &A) -> cmp::Ordering {
    a.cmp(b).reverse()
  }
}

impl<A> StaticTypeRefComparator<A> for Descend
where
  A: ?Sized + Ord + Type + for<'a> Comparable<A::Ref<'a>>,
  for<'a> A::Ref<'a>: Ord,
{
  #[inline]
  fn compare_ref(a: &A, b: &A::Ref<'_>) -> cmp::Ordering {
    a.compare(b).reverse()
  }

  #[inline]
  fn compare_refs<'a>(a: &A::Ref<'a>, b: &A::Ref<'a>) -> cmp::Ordering {
    b.cmp(a)
  }
}

impl<A, Q> StaticQueryComparator<A, Q> for Descend
where
  A: ?Sized + Ord + Comparable<Q>,
  Q: ?Sized,
{
  #[inline]
  fn query_compare(a: &A, b: &Q) -> cmp::Ordering {
    a.compare(b).reverse()
  }
}

impl<A, Q> StaticTypeRefQueryComparator<A, Q> for Descend
where
  A: ?Sized + Ord + Type + for<'a> Comparable<A::Ref<'a>>,
  for<'a> A::Ref<'a>: Comparable<Q> + Ord,
  Q: ?Sized,
{
  #[inline]
  fn query_compare_ref(a: &A::Ref<'_>, b: &Q) -> cmp::Ordering {
    a.compare(b).reverse()
  }
}

impl<A> Equivalentor<A> for Descend
where
  Descend: StaticEquivalentor<A>,
  A: ?Sized,
{
  #[inline]
  fn equivalent(&self, a: &A, b: &A) -> bool {
    <Descend as StaticEquivalentor<A>>::equivalent(a, b)
  }
}

impl<Q, A> QueryEquivalentor<A, Q> for Descend
where
  Q: ?Sized,
  A: ?Sized,
  Descend: StaticQueryEquivalentor<A, Q>,
{
  #[inline]
  fn query_equivalent(&self, a: &A, b: &Q) -> bool {
    <Descend as StaticQueryEquivalentor<A, Q>>::query_equivalent(a, b)
  }
}

impl<A> TypeRefEquivalentor<A> for Descend
where
  Descend: StaticTypeRefEquivalentor<A>,
  A: Type,
  A: ?Sized,
{
  #[inline]
  fn equivalent_ref(&self, a: &A, b: &A::Ref<'_>) -> bool {
    <Descend as StaticTypeRefEquivalentor<A>>::equivalent_ref(a, b)
  }

  #[inline]
  fn equivalent_refs<'a>(&self, a: &A::Ref<'a>, b: &A::Ref<'a>) -> bool {
    <Descend as StaticTypeRefEquivalentor<A>>::equivalent_refs(a, b)
  }
}

impl<Q, A> TypeRefQueryEquivalentor<A, Q> for Descend
where
  Q: ?Sized,
  Descend: StaticTypeRefQueryEquivalentor<A, Q>,
  A: ?Sized,
  A: Type,
{
  #[inline]
  fn query_equivalent_ref(&self, a: &A::Ref<'_>, b: &Q) -> bool {
    <Descend as StaticTypeRefQueryEquivalentor<A, Q>>::query_equivalent_ref(a, b)
  }
}

impl<A> Comparator<A> for Descend
where
  Descend: StaticComparator<A>,
  A: ?Sized,
{
  #[inline]
  fn compare(&self, a: &A, b: &A) -> cmp::Ordering {
    <Descend as StaticComparator<A>>::compare(a, b)
  }
}

impl<Q, A> QueryComparator<A, Q> for Descend
where
  Q: ?Sized,
  Descend: StaticQueryComparator<A, Q>,
  A: ?Sized,
{
  #[inline]
  fn query_compare(&self, a: &A, b: &Q) -> cmp::Ordering {
    <Descend as StaticQueryComparator<A, Q>>::query_compare(a, b)
  }
}

impl<A> TypeRefComparator<A> for Descend
where
  Descend: StaticTypeRefComparator<A>,
  A: Type,
  A: ?Sized,
{
  #[inline]
  fn compare_ref(&self, a: &A, b: &A::Ref<'_>) -> cmp::Ordering {
    <Descend as StaticTypeRefComparator<A>>::compare_ref(a, b)
  }

  #[inline]
  fn compare_refs<'a>(&self, a: &A::Ref<'a>, b: &A::Ref<'a>) -> cmp::Ordering {
    <Descend as StaticTypeRefComparator<A>>::compare_refs(a, b)
  }
}

impl<Q, A> TypeRefQueryComparator<A, Q> for Descend
where
  Q: ?Sized,
  Descend: StaticTypeRefQueryComparator<A, Q>,
  A: ?Sized,
  A: Type,
{
  #[inline]
  fn query_compare_ref(&self, a: &A::Ref<'_>, b: &Q) -> cmp::Ordering {
    <Descend as StaticTypeRefQueryComparator<A, Q>>::query_compare_ref(a, b)
  }
}
