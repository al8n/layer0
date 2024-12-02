use core::{borrow::Borrow, cmp::{self, Ordering, Reverse}, ops::{Bound, RangeBounds}};

/// Statefull Comparator for bytes.
pub trait BytesComparator {
  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent(&self, a: &[u8], b: &[u8]) -> bool;

  /// Compare `a` to `b` and return their ordering.
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering;
}

/// Stateless equivalence trait for bytes.
pub trait StaticBytesComparator {
  /// Compare `a` to `b` and return `true` if they are equal.
  fn equivalent(a: &[u8], b: &[u8]) -> bool;

  /// Compare `a` to `b` and return their ordering.
  fn compare(a: &[u8], b: &[u8]) -> cmp::Ordering;
}

/// `BytesRangeComparator` is implemented as an extention to `BytesComparator` to
/// allow for comparison of items with range bounds.
pub trait BytesRangeComparator<Q: ?Sized>: BytesComparator {
  /// Returns `true` if `item` is contained in the range.
  #[inline]
  fn compare_contains<R>(&self, range: &R, item: &[u8]) -> bool
  where
    R: ?Sized + RangeBounds<Q>,
    Q: Borrow<[u8]>,
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

impl<C, Q> BytesRangeComparator<Q> for C
where
  C: BytesComparator,
  Q: ?Sized,
{
}

impl<T> BytesComparator for T
where
  T: StaticBytesComparator,
{
  #[inline]
  fn equivalent(&self, a: &[u8], b: &[u8]) -> bool {
    T::equivalent(a, b)
  }

  #[inline]
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering {
    T::compare(a, b)
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  macro_rules! impl_traits {
    ($($ty:ty),+$(,)?) => {
      $(
        impl<C> BytesComparator for $ty
        where
          C: BytesComparator,
        {
          #[inline]
          fn equivalent(&self, a: &[u8], b: &[u8]) -> bool
          {
            (**self).equivalent(a, b)
          }

          #[inline]
          fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering
          {
            (**self).compare(a, b)
          }
        }
      )*
    };
  }

  impl_traits!(std::sync::Arc<C>, std::rc::Rc<C>);

  #[cfg(feature = "triomphe01")]
  impl_traits!(triomphe01::Arc<C>);
};

impl<C> BytesComparator for Reverse<C>
where
  C: BytesComparator,
{
  #[inline]
  fn equivalent(&self, a: &[u8], b: &[u8]) -> bool {
    self.0.equivalent(a, b)
  }

  #[inline]
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering {
    self.0.compare(a, b).reverse()
  }
}

impl BytesComparator for super::Ascend<[u8]> {
  #[inline]
  fn equivalent(&self, a: &[u8], b: &[u8]) -> bool {
    a == b
  }

  #[inline]
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering {
    a.cmp(b)
  }
}

impl BytesComparator for super::Descend<[u8]> {
  #[inline]
  fn equivalent(&self, a: &[u8], b: &[u8]) -> bool {
    a == b
  }

  #[inline]
  fn compare(&self, a: &[u8], b: &[u8]) -> cmp::Ordering {
    b.cmp(a)
  }
}
