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

impl<A> StaticTypeRefEquivalentor<A> for Ascend
where
  A: ?Sized + Eq + Type + for<'a> Equivalent<A::Ref<'a>>,
  for<'a> A::Ref<'a>: Eq,
{
  #[inline]
  fn equivalent_ref(a: &A, b: &A::Ref<'_>) -> bool {
    a.equivalent(b)
  }

  #[inline]
  fn equivalent_refs<'b>(a: &A::Ref<'b>, b: &A::Ref<'b>) -> bool {
    a == b
  }
}

impl<A, Q> StaticQueryEquivalentor<A, Q> for Ascend
where
  A: Eq + Equivalent<Q> + ?Sized,
  Q: ?Sized,
{
  #[inline]
  fn query_equivalent(a: &A, b: &Q) -> bool {
    a.equivalent(b)
  }
}

impl<A, Q> StaticTypeRefQueryEquivalentor<A, Q> for Ascend
where
  A: ?Sized + Eq + Type + for<'a> Equivalent<A::Ref<'a>>,
  for<'a> A::Ref<'a>: Equivalent<Q> + Eq,
  Q: ?Sized,
{
  #[inline]
  fn query_equivalent_ref(a: &A::Ref<'_>, b: &Q) -> bool {
    a.equivalent(b)
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

impl<A> StaticTypeRefComparator<A> for Ascend
where
  A: ?Sized + Ord + Type + for<'a> Comparable<A::Ref<'a>>,
  for<'a> A::Ref<'a>: Ord,
{
  #[inline]
  fn compare_ref(a: &A, b: &A::Ref<'_>) -> cmp::Ordering {
    a.compare(b)
  }

  #[inline]
  fn compare_refs<'a>(a: &A::Ref<'a>, b: &A::Ref<'a>) -> cmp::Ordering {
    a.cmp(b)
  }
}

impl<A, Q> StaticQueryComparator<A, Q> for Ascend
where
  A: ?Sized + Ord + Comparable<Q>,
  Q: ?Sized,
{
  #[inline]
  fn query_compare(a: &A, b: &Q) -> cmp::Ordering {
    a.compare(b)
  }
}

impl<A, Q> StaticTypeRefQueryComparator<A, Q> for Ascend
where
  A: ?Sized + Ord + Type + for<'a> Comparable<A::Ref<'a>>,
  for<'a> A::Ref<'a>: Comparable<Q> + Ord,
  Q: ?Sized,
{
  #[inline]
  fn query_compare_ref(a: &A::Ref<'_>, b: &Q) -> cmp::Ordering {
    a.compare(b)
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

impl<A> TypeRefEquivalentor<A> for Ascend
where
  Ascend: StaticTypeRefEquivalentor<A>,
  A: Type + ?Sized,
{
  #[inline]
  fn equivalent_ref(&self, a: &A, b: &A::Ref<'_>) -> bool {
    <Ascend as StaticTypeRefEquivalentor<A>>::equivalent_ref(a, b)
  }

  #[inline]
  fn equivalent_refs<'a>(&self, a: &A::Ref<'a>, b: &A::Ref<'a>) -> bool {
    <Ascend as StaticTypeRefEquivalentor<A>>::equivalent_refs(a, b)
  }
}

impl<A, Q> TypeRefQueryEquivalentor<A, Q> for Ascend
where
  Q: ?Sized,
  Ascend: StaticTypeRefQueryEquivalentor<A, Q>,
  A: Type + ?Sized,
{
  #[inline]
  fn query_equivalent_ref(&self, a: &A::Ref<'_>, b: &Q) -> bool {
    <Ascend as StaticTypeRefQueryEquivalentor<A, Q>>::query_equivalent_ref(a, b)
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

impl<A> TypeRefComparator<A> for Ascend
where
  A: Type,
  A: ?Sized,
  Ascend: StaticTypeRefComparator<A>,
{
  #[inline]
  fn compare_ref(&self, a: &A, b: &A::Ref<'_>) -> cmp::Ordering {
    <Ascend as StaticTypeRefComparator<A>>::compare_ref(a, b)
  }

  #[inline]
  fn compare_refs<'a>(&self, a: &A::Ref<'a>, b: &A::Ref<'a>) -> cmp::Ordering {
    <Ascend as StaticTypeRefComparator<A>>::compare_refs(a, b)
  }
}

impl<Q, A> TypeRefQueryComparator<A, Q> for Ascend
where
  Q: ?Sized,
  Ascend: StaticTypeRefQueryComparator<A, Q>,
  A: Type + ?Sized,
{
  #[inline]
  fn query_compare_ref(&self, a: &A::Ref<'_>, b: &Q) -> cmp::Ordering {
    <Ascend as StaticTypeRefQueryComparator<A, Q>>::query_compare_ref(a, b)
  }
}
