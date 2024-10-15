mod r#type;
use core::{
  cmp::Ordering,
  ops::{Bound, RangeBounds},
};

use equivalent::Comparable;
pub use r#type::*;

pub(super) mod comparator;

/// `ComparableRangeBounds` is implemented as an extention to `RangeBounds` to
/// allow for comparison of items with range bounds.
pub trait ComparableRangeBounds<Q: ?Sized>: RangeBounds<Q> {
  /// Returns `true` if `item` is contained in the range.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use equivalent::ComparableRangeBounds;
  ///
  /// let a = "a".to_string();
  ///
  /// assert!( ("a".."c").compare_contains(&a));
  /// assert!(!("d".."e").compare_contains("f"));
  /// ```
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
