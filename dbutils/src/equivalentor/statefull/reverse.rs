use core::cmp::{self, Reverse};

use crate::types::Type;

use super::{
  Comparator, Equivalentor, QueryComparator, QueryEquivalentor, TypeRefComparator,
  TypeRefEquivalentor, TypeRefQueryComparator, TypeRefQueryEquivalentor,
};

impl<C, T> Equivalentor<T> for Reverse<C>
where
  C: Equivalentor<T>,
  T: ?Sized,
{
  #[inline]
  fn equivalent(&self, a: &T, b: &T) -> bool {
    self.0.equivalent(a, b)
  }
}

impl<C, T> TypeRefEquivalentor<T> for Reverse<C>
where
  C: TypeRefEquivalentor<T>,
  T: Type + ?Sized,
{
  #[inline]
  fn equivalent_ref(&self, a: &T, b: &T::Ref<'_>) -> bool {
    self.0.equivalent_ref(a, b)
  }

  #[inline]
  fn equivalent_refs<'a>(&self, a: &T::Ref<'a>, b: &T::Ref<'a>) -> bool {
    self.0.equivalent_refs(a, b)
  }
}

impl<C, T, Q> QueryEquivalentor<T, Q> for Reverse<C>
where
  C: QueryEquivalentor<T, Q>,
  T: ?Sized,
  Q: ?Sized,
{
  #[inline]
  fn query_equivalent(&self, a: &T, b: &Q) -> bool {
    self.0.query_equivalent(a, b)
  }
}

impl<C, T, Q> TypeRefQueryEquivalentor<T, Q> for Reverse<C>
where
  C: TypeRefQueryEquivalentor<T, Q>,
  T: Type + ?Sized,
  Q: ?Sized,
{
  #[inline]
  fn query_equivalent_ref(&self, a: &T::Ref<'_>, b: &Q) -> bool {
    self.0.query_equivalent_ref(a, b)
  }
}

impl<C, T> Comparator<T> for Reverse<C>
where
  C: Comparator<T>,
  T: ?Sized,
{
  #[inline]
  fn compare(&self, a: &T, b: &T) -> cmp::Ordering {
    self.0.compare(a, b).reverse()
  }
}

impl<C, T> TypeRefComparator<T> for Reverse<C>
where
  C: TypeRefComparator<T>,
  T: Type + ?Sized,
{
  #[inline]
  fn compare_ref(&self, a: &T, b: &T::Ref<'_>) -> cmp::Ordering {
    self.0.compare_ref(a, b).reverse()
  }

  #[inline]
  fn compare_refs<'a>(&self, a: &T::Ref<'a>, b: &T::Ref<'a>) -> cmp::Ordering {
    self.0.compare_refs(a, b).reverse()
  }
}

impl<C, T, Q> QueryComparator<T, Q> for Reverse<C>
where
  C: QueryComparator<T, Q>,
  T: ?Sized,
  Q: ?Sized,
{
  #[inline]
  fn query_compare(&self, a: &T, b: &Q) -> cmp::Ordering {
    self.0.query_compare(a, b).reverse()
  }
}

impl<C, T, Q> TypeRefQueryComparator<T, Q> for Reverse<C>
where
  C: TypeRefQueryComparator<T, Q>,
  T: Type + ?Sized,
  Q: ?Sized,
{
  #[inline]
  fn query_compare_ref(&self, a: &T::Ref<'_>, b: &Q) -> cmp::Ordering {
    self.0.query_compare_ref(a, b).reverse()
  }
}
