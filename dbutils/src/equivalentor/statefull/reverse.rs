use core::cmp::{self, Reverse};

use crate::types::Type;

use super::{
  Comparator, Equivalentor, QueryComparator, QueryEquivalentor, TypeRefComparator,
  TypeRefEquivalentor, TypeRefQueryComparator, TypeRefQueryEquivalentor,
};

impl<C> Equivalentor for Reverse<C>
where
  C: Equivalentor,
{
  type Type = C::Type;

  #[inline]
  fn equivalent(&self, a: &Self::Type, b: &Self::Type) -> bool {
    self.0.equivalent(a, b)
  }
}

impl<'a, C> TypeRefEquivalentor<'a> for Reverse<C>
where
  C: TypeRefEquivalentor<'a>,
  C::Type: Type,
{
  #[inline]
  fn equivalent_ref(&self, a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> bool {
    self.0.equivalent_ref(a, b)
  }

  #[inline]
  fn equivalent_refs(
    &self,
    a: &<Self::Type as Type>::Ref<'a>,
    b: &<Self::Type as Type>::Ref<'a>,
  ) -> bool {
    self.0.equivalent_refs(a, b)
  }
}

impl<C, Q> QueryEquivalentor<Q> for Reverse<C>
where
  C: QueryEquivalentor<Q>,
{
  #[inline]
  fn query_equivalent(&self, a: &Self::Type, b: &Q) -> bool {
    self.0.query_equivalent(a, b)
  }
}

impl<'a, C, Q> TypeRefQueryEquivalentor<'a, Q> for Reverse<C>
where
  C: TypeRefQueryEquivalentor<'a, Q>,
  C::Type: Type,
{
  #[inline]
  fn query_equivalent_ref(&self, a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> bool {
    self.0.query_equivalent_ref(a, b)
  }
}

impl<C> Comparator for Reverse<C>
where
  C: Comparator,
{
  #[inline]
  fn compare(&self, a: &Self::Type, b: &Self::Type) -> cmp::Ordering {
    self.0.compare(a, b).reverse()
  }
}

impl<'a, C> TypeRefComparator<'a> for Reverse<C>
where
  C: TypeRefComparator<'a>,
  C::Type: Type,
{
  #[inline]
  fn compare_ref(&self, a: &Self::Type, b: &<Self::Type as Type>::Ref<'a>) -> cmp::Ordering {
    self.0.compare_ref(a, b).reverse()
  }

  #[inline]
  fn compare_refs(
    &self,
    a: &<Self::Type as Type>::Ref<'a>,
    b: &<Self::Type as Type>::Ref<'a>,
  ) -> cmp::Ordering {
    self.0.compare_refs(a, b).reverse()
  }
}

impl<C, Q> QueryComparator<Q> for Reverse<C>
where
  C: QueryComparator<Q>,
{
  #[inline]
  fn query_compare(&self, a: &Self::Type, b: &Q) -> cmp::Ordering {
    self.0.query_compare(a, b).reverse()
  }
}

impl<'a, C, Q> TypeRefQueryComparator<'a, Q> for Reverse<C>
where
  C: TypeRefQueryComparator<'a, Q>,
  C::Type: Type,
{
  #[inline]
  fn query_compare_ref(&self, a: &<Self::Type as Type>::Ref<'a>, b: &Q) -> cmp::Ordering {
    self.0.query_compare_ref(a, b).reverse()
  }
}
