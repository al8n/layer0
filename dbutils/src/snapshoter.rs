use crate::equivalentor::{Comparator, Equivalentor};

/// Validate a value.
pub trait Validator<T: ?Sized> {
  /// Returns `true` if the value is valid.
  fn validate(&self, value: &T) -> bool;
}

impl<T, V> Validator<T> for &V
where
  T: ?Sized,
  V: Validator<T>,
{
  #[inline]
  fn validate(&self, value: &T) -> bool {
    V::validate(self, value)
  }
}

/// Entry absbstrations
pub trait Entry {
  /// The key type of the entry.
  type Key: ?Sized;
  /// The value type of the entry.
  type Value: ?Sized;
  /// The version type of the entry.
  type Version: Ord;

  /// Returns the key of the entry.
  fn key(&self) -> &Self::Key;

  /// Returns the value of the entry.
  fn value(&self) -> &Self::Value;

  /// Returns the version of the entry.
  fn version(&self) -> Self::Version;
}

/// A trait for cursor entries.
///
/// A cursor entry is an entry that can be navigated to the next entry.
pub trait Cursor: Entry {
  /// Returns the next entry of the entry.
  fn next(&self) -> Option<Self>
  where
    Self: Sized;
}

/// A trait for cursor entries.
///
/// A cursor entry is an entry that can be navigated to the next or previous entry.
pub trait DoubleEndedCursor: Cursor {
  /// Returns the previous entry of the entry.
  fn next_back(&self) -> Option<Self>
  where
    Self: Sized;
}

/// An extension trait for `Cursor`.
///
/// This extension trait provides additional methods for `Cursor`.
///
/// The methods provided by this trait are used to navigate the cursor to the next valid entry with the maximum version.
pub trait CursorExt: Cursor {
  /// Returns the next valid entry with the maximum version.
  ///
  /// The version of the entry returned by this method is less than or equal to the given query version.
  fn next_by_version<E, K, V>(
    &self,
    version: &Self::Version,
    equivalentor: &E,
    key_validator: &K,
    value_validator: &V,
  ) -> Option<Self>
  where
    Self: Sized,
    E: Equivalentor<Self::Key>,
    K: Validator<Self::Key>,
    V: Validator<Self::Value>,
  {
    let mut curr = self.next();
    while let Some(ent) = curr {
      let curr_key = ent.key();
      // if the current version is larger than the query version, we should move next to find a smaller version.
      if ent.version().gt(version) {
        curr = ent.next();
        continue;
      }

      // if the value of the entry is not in a valid state, we should move next to find a valid entry.
      if !value_validator.validate(ent.value()) {
        let mut next = ent.next();
        loop {
          match next {
            None => return None,
            Some(next_ent) => {
              // if next's key is different from the current key, we should break the loop
              if !equivalentor.equivalent(next_ent.key(), curr_key) {
                curr = Some(next_ent);
                break;
              }

              next = next_ent.next();
            }
          }
        }

        continue;
      }

      // if the key of the entry is not valid, we should move next to find a valid entry.
      if key_validator.validate(curr_key) {
        return Some(ent);
      }

      curr = ent.next();
    }

    None
  }
}

impl<R> CursorExt for R where R: Cursor + ?Sized {}

/// An extension trait for `Cursor`.
///
/// This extension trait provides additional methods for `Cursor`.
///
/// The methods provided by this trait are used to navigate the cursor to the next valid entry with the maximum version.
pub trait DoubleEndedCursorExt: DoubleEndedCursor {
  /// Returns the prev valid entry with the maximum version.
  ///
  /// The version of the entry returned by this method is less than or equal to the given query version.
  fn next_back_by_version<E, K, V>(
    &self,
    version: &Self::Version,
    equivalentor: &E,
    key_validator: &K,
    value_validator: &V,
  ) -> Option<Self>
  where
    Self: Sized,
    E: Equivalentor<Self::Key>,
    K: Validator<Self::Key>,
    V: Validator<Self::Value>,
  {
    let mut curr = self.next_back();
    while let Some(ent) = curr {
      let curr_key = ent.key();
      if ent.version().gt(version) {
        curr = ent.next_back();
        continue;
      }

      let prev = ent.next_back();

      match prev {
        None => {
          if value_validator.validate(ent.value()) {
            // the current node is valid, we should return it.
            if key_validator.validate(curr_key) {
              return Some(ent);
            }
          }

          return None;
        }
        Some(prev) => {
          // At this point, prev is not null and not the head.
          // if the prev's version is greater than the query version or the prev's key is different from the current key,
          // we should try to return the current node.
          let prev_key = prev.key();
          if (prev.version().gt(version) || !equivalentor.equivalent(curr_key, prev_key))
            && value_validator.validate(ent.value())
            && key_validator.validate(curr_key)
          {
            return Some(ent);
          }

          curr = Some(prev);
        }
      }
    }

    None
  }
}

impl<R> DoubleEndedCursorExt for R where R: DoubleEndedCursor + ?Sized {}

/// Initializor trait
pub trait Initializor<E: ?Sized> {
  /// Initialize the initial value.
  ///
  /// The implementation of this method should return the same initial value when it is called multiple times.
  fn init(&self) -> Option<E>
  where
    E: Sized;
}

impl<I, E> Initializor<E> for &I
where
  I: Initializor<E> + ?Sized,
{
  #[inline]
  fn init(&self) -> Option<E> {
    I::init(self)
  }
}

/// An iterator wrapper on any iterator yielding [`Entry`].
///
/// By using the iterator wrapper, the iterator will yield [`Entry`]s with the same key only once (the entry with maximum version will be yield for the same key).
pub struct Iter<'a, HI, TI, E, C, K, V>
where
  HI: ?Sized,
  TI: ?Sized,
  C: ?Sized,
  E: Entry,
  K: ?Sized,
  V: ?Sized,
{
  comparator: &'a C,
  key_validator: &'a K,
  value_validator: &'a V,
  head_initializor: &'a HI,
  tail_initializor: &'a TI,
  tail: Option<E>,
  head: Option<E>,
  query_version: E::Version,
}

impl<'a, HI, TI, E, C, K, V> Iter<'a, HI, TI, E, C, K, V>
where
  HI: ?Sized,
  TI: ?Sized,
  C: ?Sized,
  E: Entry,
  K: ?Sized,
  V: ?Sized,
{
  /// Create a new iterator wrapper.
  #[inline]
  pub const fn new(
    version: E::Version,
    head_init: &'a HI,
    tail_init: &'a TI,
    comparator: &'a C,
    key_validator: &'a K,
    value_validator: &'a V,
  ) -> Self {
    Self {
      head_initializor: head_init,
      tail_initializor: tail_init,
      comparator,
      key_validator,
      value_validator,
      head: None,
      tail: None,
      query_version: version,
    }
  }
}

impl<HI, TI, E, C, K, V> Iterator for Iter<'_, HI, TI, E, C, K, V>
where
  C: Comparator<E::Key>,
  K: Validator<E::Key>,
  V: Validator<E::Value>,
  HI: Initializor<E>,
  TI: Initializor<E>,
  E: Cursor + Clone,
{
  type Item = E;

  fn next(&mut self) -> Option<Self::Item> {
    let mut next_head = match self.head.as_ref() {
      Some(head) => head.next(),
      None => self.head_initializor.init(),
    };

    next_head = next_head.and_then(|h| {
      h.next_by_version(
        &self.query_version,
        &self.comparator,
        &self.key_validator,
        &self.value_validator,
      )
    });

    match (next_head, &self.tail) {
      (Some(next), Some(t))
        if self
          .comparator
          .compare(next.key(), t.key())
          .then_with(|| t.version().cmp(&next.version()))
          .is_ge() =>
      {
        self.head = Some(next);
        None
      }
      (Some(next), _) => {
        self.head = Some(next);
        self.head.clone()
      }
      (None, _) => {
        self.head = None;
        None
      }
    }
  }
}

impl<HI, TI, E, C, K, V> DoubleEndedIterator for Iter<'_, HI, TI, E, C, K, V>
where
  C: Comparator<E::Key>,
  K: Validator<E::Key>,
  V: Validator<E::Value>,
  HI: Initializor<E>,
  TI: Initializor<E>,
  E: DoubleEndedCursor + Clone,
{
  fn next_back(&mut self) -> Option<Self::Item> {
    let mut next_tail = match self.tail.as_ref() {
      Some(tail) => tail.next_back(),
      None => self.tail_initializor.init(),
    };

    next_tail = next_tail.and_then(|t| {
      t.next_back_by_version(
        &self.query_version,
        &self.comparator,
        &self.key_validator,
        &self.value_validator,
      )
    });

    match (&self.head, next_tail) {
      (Some(h), Some(next))
        if self
          .comparator
          .compare(h.key(), next.key())
          .then_with(|| h.version().cmp(&next.version()))
          .is_ge() =>
      {
        self.tail = Some(next);
        None
      }
      (_, Some(next)) => {
        self.tail = Some(next);
        self.tail.clone()
      }
      (_, None) => {
        self.tail = None;
        None
      }
    }
  }
}
