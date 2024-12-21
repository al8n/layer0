#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]

use core::ops::{Bound, RangeBounds};

pub use dbutils::equivalentor;
use equivalentor::{Ascend, Equivalentor};

/// Provides deduplication functionality for iterators and ranges.
///
/// This module ensures that:
/// - Only entries with unique keys are yielded
/// - For entries with the same key, the entry with the maximum version is returned
/// - Iteration respects the specified query version limit
/// - Entries can be filtered based on custom key and value validators
///
/// # Key Features
/// - Eliminates duplicate keys during iteration
/// - Filters entries based on a version constraint
/// - Flexible validation of keys and values
/// - Ensures iteration only includes entries meeting specified criteria
pub mod dedup;

/// Provides validation functionality for iterators and ranges.
///
/// This module ensures that:
/// - Only entries less than or equal to the specified query version are yielded
/// - Entries can be filtered based on custom key and value validators
///
/// # Key Features
/// - Version-based filtering of entries
/// - Flexible validation of keys and values
/// - Ensures iteration only includes entries meeting specified criteria
pub mod valid;

mod sealed;

/// A trait for types that can be finalized to a `Range`.
pub trait ToRange<Q, R, E>: sealed::SealedRange<Q, R, E>
where
  E: ?Sized,
  Q: ?Sized,
  R: RangeBounds<Q>,
{
}

impl<Q, R, E, T> ToRange<Q, R, E> for T
where
  E: ?Sized,
  Q: ?Sized,
  R: RangeBounds<Q>,
  T: sealed::SealedRange<Q, R, E>,
{
}

/// A trait for types that can be finalized to a `Range`.
pub trait ToIter<E>: sealed::SealedIter<E>
where
  E: ?Sized,
{
}

impl<E, T> ToIter<E> for T
where
  E: ?Sized,
  T: sealed::SealedIter<E>,
{
}

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

/// A no-op validator.
pub struct NoopValidator;

impl<T: ?Sized> Validator<T> for NoopValidator {
  #[inline(always)]
  fn validate(&self, _: &T) -> bool {
    true
  }
}

/// Any validator.
pub struct AnyValidator<F>(pub F);

impl<F, T: ?Sized> Validator<T> for AnyValidator<F>
where
  F: Fn(&T) -> bool,
{
  #[inline]
  fn validate(&self, value: &T) -> bool {
    (self.0)(value)
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

/// A trait for rewinding between the front and back.
pub trait Rewindable {
  /// The entry can be yielded by the seeker.
  type Entry;

  /// Returns the first entry.
  fn first(&self) -> Option<Self::Entry>;

  /// Returns the last entry.
  fn last(&self) -> Option<Self::Entry>;
}

/// A trait for seeking between entries.
pub trait Seekable<Q: ?Sized> {
  /// The entry can be yielded by the seeker.
  type Entry;

  /// Returns the lower bound of the entry.
  fn lower_bound(&self, bound: Bound<&Q>) -> Option<Self::Entry>;

  /// Returns the upper bound of the entry.
  fn upper_bound(&self, bound: Bound<&Q>) -> Option<Self::Entry>;
}

/// Extension methods for single-directional cursors with additional validation and deduplication capabilities.
///
/// This trait adds advanced traversal methods to the base [`Cursor`] trait, allowing for:
/// - Version-based filtering
/// - Key and value validation
/// - Deduplication of entries
pub trait CursorExt: Cursor {
  /// Advances to the next entry that is valid according to the specified version and validators.
  fn next_valid<E, K, V>(
    &self,
    version: &Self::Version,
    key_validator: &K,
    value_validator: &V,
  ) -> Option<Self>
  where
    Self: Sized,
    E: Equivalentor<Self::Key>,
    K: Validator<Self::Key>,
    V: Validator<Self::Value>,
  {
    let curr = self.next();
    next_valid(curr, version, key_validator, value_validator)
  }

  /// Advances to the next entry, filtering by version and deduplicating entries with the same key.
  ///
  /// - Skips entries that do not meet version or validation criteria.
  /// - When multiple entries exist for the same key, returns the entry with the maximum version.
  fn next_dedup<E, K, V>(
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
    let curr = self.next();
    next_dedup(curr, version, equivalentor, key_validator, value_validator)
  }
}

impl<R> CursorExt for R where R: Cursor + ?Sized {}

/// Extension methods for bi-directional cursors with additional validation and deduplication capabilities.
///
/// This trait adds advanced traversal methods to the base [`DoubleEndedCursor`] trait,
/// providing similar functionality to [`CursorExt`] but for backwards traversal.
pub trait DoubleEndedCursorExt: DoubleEndedCursor {
  /// Moves backwards to the next entry that is valid according to the specified version and validators.
  fn next_back_valid<E, K, V>(
    &self,
    version: &Self::Version,
    key_validator: &K,
    value_validator: &V,
  ) -> Option<Self>
  where
    Self: Sized,
    E: Equivalentor<Self::Key>,
    K: Validator<Self::Key>,
    V: Validator<Self::Value>,
  {
    let curr = self.next();
    next_back_valid(curr, version, key_validator, value_validator)
  }

  /// Moves backwards to the next entry, filtering by version and deduplicating entries with the same key.
  ///
  /// - Skips entries that do not meet version or validation criteria.
  /// - When multiple entries exist for the same key, returns the entry with the maximum version when moving backwards.
  fn next_back_dedup<E, K, V>(
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
    let curr = self.next_back();
    next_back_dedup(curr, version, equivalentor, key_validator, value_validator)
  }
}

impl<R> DoubleEndedCursorExt for R where R: DoubleEndedCursor + ?Sized {}

/// The builder for creating an iterator.
pub struct Builder<I, C = Ascend, K = NoopValidator, V = NoopValidator> {
  comparator: C,
  key_validator: K,
  value_validator: V,
  initializor: I,
}

impl<I, C, K, V> Default for Builder<I, C, K, V>
where
  C: Default,
  K: Default,
  V: Default,
  I: Default,
{
  fn default() -> Self {
    Self {
      comparator: Default::default(),
      key_validator: Default::default(),
      value_validator: Default::default(),
      initializor: Default::default(),
    }
  }
}

impl<I> Builder<I> {
  /// Creates a new builder with default values.
  #[inline]
  pub const fn new(init: I) -> Self {
    Self {
      comparator: Ascend,
      key_validator: NoopValidator,
      value_validator: NoopValidator,
      initializor: init,
    }
  }
}

impl<I, C, K, V> Builder<I, C, K, V> {
  /// Sets the comparator for the builder.
  #[inline]
  pub fn with_comparator<NC>(self, comparator: NC) -> Builder<I, NC, K, V> {
    Builder {
      comparator,
      key_validator: self.key_validator,
      value_validator: self.value_validator,
      initializor: self.initializor,
    }
  }

  /// Sets the key validator for the builder.
  #[inline]
  pub fn with_key_validator<NK>(self, key_validator: NK) -> Builder<I, C, NK, V> {
    Builder {
      comparator: self.comparator,
      key_validator,
      value_validator: self.value_validator,
      initializor: self.initializor,
    }
  }

  /// Sets the value validator for the builder.
  #[inline]
  pub fn with_value_validator<NV>(self, value_validator: NV) -> Builder<I, C, K, NV> {
    Builder {
      comparator: self.comparator,
      key_validator: self.key_validator,
      value_validator,
      initializor: self.initializor,
    }
  }

  /// Finalizes the builder into an iterator.
  #[inline]
  pub fn iter<E, F>(self, version: E::Version) -> F
  where
    E: Entry,
    F: ToIter<E, Initializor = I, Comparator = C, KeyValidator = K, ValueValidator = V>,
    I: Rewindable<Entry = E>,
  {
    F::new(version, self)
  }

  /// Finalizes the builder into a range.
  #[inline]
  pub fn range<E, F, Q, R>(self, version: E::Version, range: R) -> F
  where
    R: RangeBounds<Q>,
    Q: ?Sized,
    E: Entry,
    F: ToRange<Q, R, E, Initializor = I, Comparator = C, KeyValidator = K, ValueValidator = V>,
    I: Seekable<Q, Entry = E>,
  {
    F::range(version, range, self)
  }
}

fn next_dedup<ENT, E, K, V>(
  mut curr: Option<ENT>,
  version: &ENT::Version,
  equivalentor: &E,
  key_validator: &K,
  value_validator: &V,
) -> Option<ENT>
where
  ENT: Sized + Entry + Cursor,
  E: Equivalentor<ENT::Key>,
  K: Validator<ENT::Key>,
  V: Validator<ENT::Value>,
{
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

fn next_back_dedup<ENT, E, K, V>(
  mut curr: Option<ENT>,
  version: &ENT::Version,
  equivalentor: &E,
  key_validator: &K,
  value_validator: &V,
) -> Option<ENT>
where
  ENT: Sized + Entry + DoubleEndedCursor,
  E: Equivalentor<ENT::Key>,
  K: Validator<ENT::Key>,
  V: Validator<ENT::Value>,
{
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

fn next_valid<ENT, K, V>(
  mut curr: Option<ENT>,
  version: &ENT::Version,
  key_validator: &K,
  value_validator: &V,
) -> Option<ENT>
where
  ENT: Sized + Entry + Cursor,
  K: Validator<ENT::Key>,
  V: Validator<ENT::Value>,
{
  while let Some(ent) = curr {
    let curr_key = ent.key();
    if ent.version().gt(version) {
      curr = ent.next();
      continue;
    }

    // if the key of the entry is not valid, we should move next to find a valid entry.
    if key_validator.validate(curr_key) && value_validator.validate(ent.value()) {
      return Some(ent);
    }

    curr = ent.next();
  }

  None
}

fn next_back_valid<ENT, K, V>(
  mut curr: Option<ENT>,
  version: &ENT::Version,
  key_validator: &K,
  value_validator: &V,
) -> Option<ENT>
where
  ENT: Sized + Entry + DoubleEndedCursor,
  K: Validator<ENT::Key>,
  V: Validator<ENT::Value>,
{
  while let Some(ent) = curr {
    let curr_key = ent.key();
    if ent.version().gt(version) {
      curr = ent.next_back();
      continue;
    }

    // if the key of the entry is not valid, we should move next to find a valid entry.
    if key_validator.validate(curr_key) && value_validator.validate(ent.value()) {
      return Some(ent);
    }

    curr = ent.next_back();
  }

  None
}
