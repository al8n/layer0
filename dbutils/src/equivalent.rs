pub use equivalent::{Comparable, Equivalent};

use core::{
  cmp::Ordering,
  ops::{Bound, RangeBounds},
};

/// `ComparableRangeBounds` is implemented as an extention to `RangeBounds` to
/// allow for comparison of items with range bounds.
pub trait ComparableRangeBounds<Q: ?Sized>: RangeBounds<Q> {
  /// Returns `true` if `item` is contained in the range.
  fn compare_contains<K>(&self, item: &K) -> bool
  where
    Q: Comparable<K>,
    K: ?Sized,
  {
    (match self.start_bound() {
      Bound::Included(start) => start.compare(item) != Ordering::Greater,
      Bound::Excluded(start) => start.compare(item) == Ordering::Less,
      Bound::Unbounded => true,
    }) && (match self.end_bound() {
      Bound::Included(end) => end.compare(item) != Ordering::Less,
      Bound::Excluded(end) => end.compare(item) == Ordering::Greater,
      Bound::Unbounded => true,
    })
  }
}

impl<R, T> ComparableRangeBounds<T> for R
where
  R: ?Sized + RangeBounds<T>,
  T: ?Sized,
{
}
