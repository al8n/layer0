use core::{
  marker::PhantomData,
  ops::{Bound, RangeBounds},
};

use dbutils::equivalentor::{Comparator, QueryComparator};

use crate::{
  next_back_valid, next_valid, sealed::SealedRange, Builder, Cursor, DoubleEndedCursor, Entry,
  Seekable, Validator,
};

/// An iterator wrapper on any iterator yielding [`Entry`].
///
/// By using the iterator wrapper, the iterator will yield [`Entry`]s with the same key only once (the entry with maximum version will be yield for the same key).
pub struct Range<R, Q, S, E, C, K, V>
where
  E: Entry,
  Q: ?Sized,
  R: RangeBounds<Q>,
  S: Seekable<Q, Entry = E>,
{
  comparator: C,
  key_validator: K,
  value_validator: V,
  seeker: S,
  tail: Option<E>,
  head: Option<E>,
  query_version: E::Version,
  range: R,
  _q: PhantomData<Q>,
}

impl<R, Q, S, E, C, K, V> SealedRange<Q, R, E> for Range<R, Q, S, E, C, K, V>
where
  E: Entry,
  Q: ?Sized,
  R: RangeBounds<Q>,
  S: Seekable<Q, Entry = E>,
{
  type Initializor = S;

  type KeyValidator = K;

  type ValueValidator = V;

  type Comparator = C;

  fn range(
    version: E::Version,
    range: R,
    builder: Builder<Self::Initializor, Self::Comparator, Self::KeyValidator, Self::ValueValidator>,
  ) -> Self
  where
    E: Entry,
    Self: Sized,
    Self::Initializor: Seekable<Q, Entry = E>,
  {
    Self {
      seeker: builder.initializor,
      comparator: builder.comparator,
      key_validator: builder.key_validator,
      value_validator: builder.value_validator,
      head: None,
      tail: None,
      query_version: version,
      range,
      _q: PhantomData,
    }
  }
}

impl<R, Q, S, E, C, K, V> Range<R, Q, S, E, C, K, V>
where
  E: Entry,
  Q: ?Sized,
  R: RangeBounds<Q>,
  S: Seekable<Q, Entry = E>,
{
  /// Returns the query version of the iterator.
  #[inline]
  pub const fn query_version(&self) -> &E::Version {
    &self.query_version
  }

  /// Returns the current head of the iterator.
  #[inline]
  pub const fn head(&self) -> Option<&E> {
    self.head.as_ref()
  }

  /// Returns the current tail of the iterator.
  #[inline]
  pub const fn tail(&self) -> Option<&E> {
    self.tail.as_ref()
  }

  /// Returns the range.
  #[inline]
  pub const fn range(&self) -> &R {
    &self.range
  }
}

impl<R, Q, S, E, C, K, V> Iterator for Range<R, Q, S, E, C, K, V>
where
  K: Validator<E::Key>,
  V: Validator<E::Value>,
  S: Seekable<Q, Entry = E>,
  E: Cursor + Clone,
  C: QueryComparator<E::Key, Q>,
  Q: ?Sized,
  R: RangeBounds<Q>,
{
  type Item = E;

  fn next(&mut self) -> Option<Self::Item> {
    let next_head = match self.head.as_ref() {
      Some(head) => head.next(),
      None => self.seeker.lower_bound(self.range.start_bound()),
    };

    self.head = next_valid(
      next_head,
      &self.query_version,
      &self.key_validator,
      &self.value_validator,
    );

    if let Some(ref h) = self.head {
      match &self.tail {
        Some(t) => {
          let bound = Bound::Excluded(t.key());
          if !below_upper_bound(&self.comparator, &bound, h.key()) {
            self.head = None;
            self.tail = None;
          }
        }
        None => {
          let bound = self.range.end_bound();
          if !below_upper_bound_compare(&self.comparator, &bound, h.key()) {
            self.head = None;
            self.tail = None;
          }
        }
      }
    }

    self.head.clone()
  }
}

impl<R, Q, S, E, C, K, V> DoubleEndedIterator for Range<R, Q, S, E, C, K, V>
where
  K: Validator<E::Key>,
  V: Validator<E::Value>,
  S: Seekable<Q, Entry = E>,
  E: Entry + DoubleEndedCursor + Clone,
  C: QueryComparator<E::Key, Q>,
  Q: ?Sized,
  R: RangeBounds<Q>,
{
  fn next_back(&mut self) -> Option<Self::Item> {
    let next_tail = match self.tail.as_ref() {
      Some(tail) => tail.next_back(),
      None => self.seeker.upper_bound(self.range.end_bound()),
    };

    self.tail = next_back_valid(
      next_tail,
      &self.query_version,
      &self.key_validator,
      &self.value_validator,
    );

    if let Some(ref t) = self.tail {
      match &self.head {
        Some(h) => {
          let bound = Bound::Excluded(h.key());
          if !above_lower_bound(&self.comparator, &bound, t.key()) {
            self.head = None;
            self.tail = None;
          }
        }
        None => {
          let bound = self.range.start_bound();
          if !above_lower_bound_compare(&self.comparator, &bound, t.key()) {
            self.head = None;
            self.tail = None;
          }
        }
      }
    }

    self.tail.clone()
  }
}

/// Helper function to check if a value is above a lower bound
fn above_lower_bound_compare<C, V, T>(cmp: &C, bound: &Bound<&T>, other: &V) -> bool
where
  V: ?Sized,
  T: ?Sized,
  C: QueryComparator<V, T>,
{
  match *bound {
    Bound::Unbounded => true,
    Bound::Included(key) => cmp.query_compare(other, key).is_ge(),
    Bound::Excluded(key) => cmp.query_compare(other, key).is_gt(),
  }
}

/// Helper function to check if a value is above a lower bound
fn above_lower_bound<C, K>(cmp: &C, bound: &Bound<&K>, other: &K) -> bool
where
  C: Comparator<K>,
  K: ?Sized,
{
  match *bound {
    Bound::Unbounded => true,
    Bound::Included(key) => cmp.compare(key, other).is_le(),
    Bound::Excluded(key) => cmp.compare(key, other).is_lt(),
  }
}

/// Helper function to check if a value is below an upper bound
fn below_upper_bound_compare<C, V, T>(cmp: &C, bound: &Bound<&T>, other: &V) -> bool
where
  V: ?Sized,
  T: ?Sized,
  C: QueryComparator<V, T>,
{
  match *bound {
    Bound::Unbounded => true,
    Bound::Included(key) => cmp.query_compare(other, key).is_le(),
    Bound::Excluded(key) => cmp.query_compare(other, key).is_lt(),
  }
}

/// Helper function to check if a value is below an upper bound
fn below_upper_bound<C, K>(cmp: &C, bound: &Bound<&K>, other: &K) -> bool
where
  C: Comparator<K>,
  K: ?Sized,
{
  match *bound {
    Bound::Unbounded => true,
    Bound::Included(key) => cmp.compare(key, other).is_ge(),
    Bound::Excluded(key) => cmp.compare(key, other).is_gt(),
  }
}
