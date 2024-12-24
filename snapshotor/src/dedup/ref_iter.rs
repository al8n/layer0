use dbutils::equivalentor::Comparator;

use crate::{
  next_back_dedup, next_dedup, sealed::SealedIter, Builder, Cursor, DoubleEndedCursor, Entry,
  Rewindable, Validator,
};

struct RefIterKeyValidator<'a, C, E, V>
where
  C: Comparator<E::Key>,
  E: Entry,
  V: Validator<E::Key>,
{
  key_validator: &'a V,
  comparator: &'a C,
  last: Option<&'a E::Key>,
}

impl<'a, C, E, V> RefIterKeyValidator<'a, C, E, V>
where
  C: Comparator<E::Key>,
  E: Entry,
  V: Validator<E::Key>,
{
  #[inline]
  const fn new(key_validator: &'a V, comparator: &'a C, last: Option<&'a E::Key>) -> Self {
    Self {
      key_validator,
      comparator,
      last,
    }
  }
}

impl<C, E, V> Validator<E::Key> for RefIterKeyValidator<'_, C, E, V>
where
  C: Comparator<E::Key>,
  E: Entry,
  V: Validator<E::Key>,
{
  #[inline]
  fn validate(&self, key: &E::Key) -> bool {
    let same = if let Some(last) = self.last {
      self.comparator.equivalent(key, last)
    } else {
      false
    };

    !same && self.key_validator.validate(key)
  }
}

/// An iterator wrapper on any iterator yielding [`Entry`].
///
/// By using the iterator wrapper, the iterator will yield [`Entry`]s with the same key only once (the entry with maximum version will be yield for the same key).
pub struct RefIter<'a, E, R, C, K, V>
where
  E: Entry,
{
  comparator: &'a C,
  key_validator: K,
  value_validator: V,
  rewinder: R,
  tail: Option<E>,
  head: Option<E>,
  query_version: E::Version,
}

impl<'a, E, R, C, K, V> SealedIter<E> for RefIter<'a, E, R, C, K, V>
where
  E: Entry,
{
  type Initializor = R;

  type KeyValidator = K;

  type ValueValidator = V;

  type Comparator = &'a C;

  fn new(
    version: E::Version,
    builder: Builder<Self::Initializor, Self::Comparator, Self::KeyValidator, Self::ValueValidator>,
  ) -> Self
  where
    E: Entry,
  {
    Self {
      rewinder: builder.initializor,
      comparator: builder.comparator,
      key_validator: builder.key_validator,
      value_validator: builder.value_validator,
      head: None,
      tail: None,
      query_version: version,
    }
  }
}

impl<E, R, C, K, V> RefIter<'_, E, R, C, K, V>
where
  E: Entry,
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
}

impl<E, R, C, K, V> Iterator for RefIter<'_, E, R, C, K, V>
where
  C: Comparator<E::Key>,
  K: Validator<E::Key>,
  V: Validator<E::Value>,
  R: Rewindable<Entry = E>,
  E: Cursor + Clone,
{
  type Item = E;

  fn next(&mut self) -> Option<Self::Item> {
    let mut next_head = match self.head.as_ref() {
      Some(head) => head.next(),
      None => self.rewinder.first(),
    };

    let kv = RefIterKeyValidator::<C, E, K>::new(
      &self.key_validator,
      self.comparator,
      self.head.as_ref().map(|h| h.key()),
    );

    next_head = next_dedup(
      next_head,
      &self.query_version,
      &self.comparator,
      &kv,
      &self.value_validator,
    );

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

impl<E, R, C, K, V> DoubleEndedIterator for RefIter<'_, E, R, C, K, V>
where
  C: Comparator<E::Key>,
  K: Validator<E::Key>,
  V: Validator<E::Value>,
  R: Rewindable<Entry = E>,
  E: DoubleEndedCursor + Clone,
{
  fn next_back(&mut self) -> Option<Self::Item> {
    let mut next_tail = match self.tail.as_ref() {
      Some(tail) => tail.next_back(),
      None => self.rewinder.last(),
    };

    let kv = RefIterKeyValidator::<C, E, K>::new(
      &self.key_validator,
      self.comparator,
      self.tail.as_ref().map(|h| h.key()),
    );

    next_tail = next_back_dedup(
      next_tail,
      &self.query_version,
      &self.comparator,
      &kv,
      &self.value_validator,
    );

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
