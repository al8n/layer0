use {
  cheap_clone::CheapClone,
  core::{
    borrow::Borrow,
    cmp::{self, Ordering, Reverse},
    ops::{Bound, RangeBounds},
  }, equivalent::{Comparable, Equivalent},
};

/// Custom equivalence trait.
pub trait DynEquivalentor<A, B>
where
  A: ?Sized,
  B: ?Sized,
{
  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent(&self, a: &A, b: &B) -> bool;
}

/// Custom ordering trait.
pub trait DynComparator<A, B>: DynEquivalentor<A, B>
where
  A: ?Sized,
  B: ?Sized,
{
  /// Compare `a` to `b` and return their ordering.
  fn compare(&self, a: &A, b: &B) -> cmp::Ordering;
}

/// `RangeComparator` is implemented as an extention to `Comparator` to
/// allow for comparison of items with range bounds.
pub trait DynRangeComparator<A, B>: DynComparator<A, B>
where
  A: ?Sized,
  B: ?Sized,
{
  /// Returns `true` if `item` is contained in the range.
  fn compare_contains<R, Q>(&self, range: &R, item: &A) -> bool
  where
    Q: ?Sized + Borrow<B>,
    R: ?Sized + RangeBounds<Q>,
  {
    let start = match range.start_bound() {
      Bound::Included(start) => self.compare(item, start.borrow()) != Ordering::Less,
      Bound::Excluded(start) => self.compare(item, start.borrow()) == Ordering::Greater,
      Bound::Unbounded => true,
    };

    let end = match range.end_bound() {
      Bound::Included(end) => self.compare(item, end.borrow()) != Ordering::Greater,
      Bound::Excluded(end) => self.compare(item, end.borrow()) == Ordering::Less,
      Bound::Unbounded => true,
    };

    start && end
  }
}

impl<A, B, C> DynRangeComparator<A, B> for C
where
  A: ?Sized,
  B: ?Sized,
  C: DynComparator<A, B>,
{}

/// Custom equivalence trait.
///
/// Comparing to [`DynEquivalentor`], `Equivalentor` is not object-safe, but it can store some information about how to compare.
pub trait Equivalentor<A>
where
  A: ?Sized,
{
  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent<B>(&self, a: &A, b: &B) -> bool
  where
    B: ?Sized + Equivalent<A>;
}

/// Custom ordering trait.
///
/// Comparing to [`DynComparator`], `Comparator` is not object-safe, but it can store some information about how to compare.
pub trait Comparator<A>: Equivalentor<A>
where
  A: ?Sized,
{
  /// Compare `a` to `b` and return their ordering.
  fn compare<B>(&self, a: &A, b: &B) -> cmp::Ordering
  where
    B: ?Sized + Comparable<A>;
}

/// `RangeComparator` is implemented as an extention to `Comparator` to
/// allow for comparison of items with range bounds.
pub trait RangeComparator<A>: Comparator<A>
where
  A: ?Sized,
{
  /// Returns `true` if `item` is contained in the range.
  fn compare_contains<R, Q>(&self, range: &R, item: &A) -> bool
  where
    Q: ?Sized + Comparable<A>,
    R: ?Sized + RangeBounds<Q>,
  {
    let start = match range.start_bound() {
      Bound::Included(start) => self.compare(item, start.borrow()) != Ordering::Less,
      Bound::Excluded(start) => self.compare(item, start.borrow()) == Ordering::Greater,
      Bound::Unbounded => true,
    };

    let end = match range.end_bound() {
      Bound::Included(end) => self.compare(item, end.borrow()) != Ordering::Greater,
      Bound::Excluded(end) => self.compare(item, end.borrow()) == Ordering::Less,
      Bound::Unbounded => true,
    };

    start && end
  }
}

impl<A, C> RangeComparator<A> for C
where
  A: ?Sized,
  C: Comparator<A>,
{}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  macro_rules! impl_traits {
    ($($ty:ty),+$(,)?) => {
      $(
        impl<A, C: Equivalentor<A>> Equivalentor<A> for $ty {
          #[inline]
          fn equivalent<B>(&self, a: &A, b: &B) -> bool
          where
            B: ?Sized + Equivalent<A>,
          {
            (**self).equivalent(a, b)
          }
        }

        impl<A, C: Comparator<A>> Comparator<A> for $ty {
          #[inline]
          fn compare<B>(&self, a: &A, b: &B) -> cmp::Ordering
          where
            B: ?Sized + Comparable<A>,
          {
            (**self).compare(a, b)
          }
        }
      )*
    };
  }

  impl_traits!(std::sync::Arc<C>, std::rc::Rc<C>, std::boxed::Box<C>);

  #[cfg(feature = "triomphe01")]
  impl_traits!(triomphe01::Arc<C>);
};

/// Custom equivalence trait.
///
/// Comparing to [`DynEquivalentor`], `StaticEquivalentor` is not object-safe, but it does not to create a new object when comparing.
pub trait StaticEquivalentor<A, B>
where
  A: ?Sized,
  B: ?Sized,
{
  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent(a: &A, b: &B) -> bool;
}

/// Custom ordering trait.
///
/// Comparing to [`DynComparator`], `StaticComparator` is not object-safe, but it does not to create a new object when comparing.
pub trait StaticComparator<A, B>: StaticEquivalentor<A, B>
where
  A: ?Sized,
  B: ?Sized,
{
  /// Compare `a` to `b` and return their ordering.
  fn compare(a: &A, b: &B) -> cmp::Ordering;
}

/// `StaticRangeComparator` is implemented as an extention to `StaticComparator` to
/// allow for comparison of items with range bounds.
pub trait StaticRangeComparator<A, B>: StaticComparator<A, B>
where
  A: ?Sized,
  B: ?Sized,
{
  /// Returns `true` if `item` is contained in the range.
  fn compare_contains<R, Q>(range: &R, item: &A) -> bool
  where
    Q: ?Sized + Borrow<B>,
    R: ?Sized + RangeBounds<Q>,
  {
    let start = match range.start_bound() {
      Bound::Included(start) => Self::compare(item, start.borrow()) != Ordering::Less,
      Bound::Excluded(start) => Self::compare(item, start.borrow()) == Ordering::Greater,
      Bound::Unbounded => true,
    };

    let end = match range.end_bound() {
      Bound::Included(end) => Self::compare(item, end.borrow()) != Ordering::Greater,
      Bound::Excluded(end) => Self::compare(item, end.borrow()) == Ordering::Less,
      Bound::Unbounded => true,
    };

    start && end
  }
}

impl<A, B, C> StaticRangeComparator<A, B> for C
where
  A: ?Sized,
  B: ?Sized,
  C: StaticComparator<A, B>
{}

impl<A, B, C> DynEquivalentor<A, B> for C
where
  A: ?Sized,
  B: ?Sized,
  C: StaticEquivalentor<A, B>
{
  #[inline]
  fn equivalent(&self, a: &A, b: &B) -> bool {
    C::equivalent(a, b)
  }
}

impl<A, B, C> DynComparator<A, B> for C
where
  A: ?Sized,
  B: ?Sized,
  C: StaticComparator<A, B>
{
  #[inline]
  fn compare(&self, a: &A, b: &B) -> cmp::Ordering {
    C::compare(a, b)
  }
}

/// Ascend is a comparator that compares byte slices in ascending order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ascend;

impl<A, B> StaticEquivalentor<A, B> for Ascend
where
  A: ?Sized,
  B: ?Sized + Equivalent<A>,
{
  #[inline]
  fn equivalent(a: &A, b: &B) -> bool {
    Equivalent::equivalent(b, a)
  }
}

impl<A, B> StaticComparator<A, B> for Ascend
where
  A: ?Sized,
  B: ?Sized + Comparable<A>,
{
  #[inline]
  fn compare(a: &A, b: &B) -> cmp::Ordering {
    Comparable::compare(b, a).reverse()
  }
}

impl CheapClone for Ascend {}

/// Descend is a comparator that compares byte slices in descending order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Descend;

impl<A, B> StaticEquivalentor<A, B> for Descend
where
  A: ?Sized,
  B: ?Sized + Equivalent<A>,
{
  #[inline]
  fn equivalent(a: &A, b: &B) -> bool {
    Equivalent::equivalent(b, a)
  }
}

impl<A, B> StaticComparator<A, B> for Descend
where
  A: ?Sized,
  B: ?Sized + Comparable<A>,
{
  #[inline]
  fn compare(a: &A, b: &B) -> cmp::Ordering {
    Comparable::compare(b, a)
  }
}

impl CheapClone for Descend {}

impl<A, B, C> StaticEquivalentor<A, B> for Reverse<C>
where
  A: ?Sized,
  B: ?Sized,
  C: StaticEquivalentor<A, B>,
{
  #[inline]
  fn equivalent(a: &A, b: &B) -> bool {
    C::equivalent(a, b)
  }
}

impl<A, B, C> StaticComparator<A, B> for Reverse<C>
where
  A: ?Sized,
  B: ?Sized + Comparable<A>,
  C: StaticComparator<A, B>,
{
  #[inline]
  fn compare(a: &A, b: &B) -> cmp::Ordering {
    C::compare(a, b).reverse()
  }
}

#[cfg(test)]
mod tests {
  use core::cmp;

  use super::{Ascend, DynComparator, Descend, DynEquivalentor, DynRangeComparator, Reverse};

  #[test]
  fn test_desc() {
    let desc = Descend;
    assert_eq!(desc.compare(b"abc", b"def"), cmp::Ordering::Greater);
    assert_eq!(desc.compare(b"def", b"abc"), cmp::Ordering::Less);
    assert_eq!(desc.compare(b"abc", b"abc"), cmp::Ordering::Equal);
    assert!(DynRangeComparator::<[u8], [u8]>::compare_contains(&desc, &("b".as_bytes().."a".as_bytes()), "b".as_bytes()));
  }

  #[test]
  fn test_desc_reverse() {
    let desc = Reverse(Descend);
    assert_eq!(desc.compare(b"abc", b"def"), cmp::Ordering::Less);
    assert_eq!(desc.compare(b"def", b"abc"), cmp::Ordering::Greater);
    assert_eq!(desc.compare(b"abc", b"abc"), cmp::Ordering::Equal);

    assert!(DynRangeComparator::<[u8], [u8]>::compare_contains(&desc, &("a".as_bytes()..="d".as_bytes()), "b".as_bytes()));
  }

  #[test]
  fn test_asc() {
    let asc = Ascend;
    assert_eq!(asc.compare(b"abc", b"def"), cmp::Ordering::Less);
    assert_eq!(asc.compare(b"def", b"abc"), cmp::Ordering::Greater);
    assert_eq!(asc.compare(b"abc", b"abc"), cmp::Ordering::Equal);

    assert!(DynRangeComparator::<[u8], [u8]>::compare_contains(&asc, &("a".as_bytes().."d".as_bytes()), "b".as_bytes()));
  }

  #[test]
  fn test_asc_reverse() {
    let asc = Reverse(Ascend);
    assert_eq!(asc.compare(b"abc", b"def"), cmp::Ordering::Greater);
    assert_eq!(asc.compare(b"def", b"abc"), cmp::Ordering::Less);
    assert_eq!(asc.compare(b"abc", b"abc"), cmp::Ordering::Equal);
    assert!(asc.equivalent(b"a", b"a"));
    assert!(DynRangeComparator::<[u8], [u8]>::compare_contains(&asc, &("d".as_bytes()..="a".as_bytes()), "d".as_bytes()));
  }
}
