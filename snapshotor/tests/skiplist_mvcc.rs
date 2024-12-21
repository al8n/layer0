use core::{
  cmp,
  marker::PhantomData,
  ops::{Bound, RangeBounds},
  sync::atomic::{AtomicU64, Ordering},
};
use crossbeam_skiplist::{
  equivalent::{Comparable, Equivalent},
  SkipMap as CSkipMap,
};
use dbutils::state::{Active, MaybeTombstone};

/// Errors for multiple version `SkipMap`s
#[derive(Debug, Clone)]
pub enum Error {
  /// Returned when trying to insert an entry with a version that already been discarded.
  AlreadyDiscarded(u64),
}

impl core::fmt::Display for Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::AlreadyDiscarded(version) => write!(
        f,
        "version({version}) has already been discarded by compaction"
      ),
    }
  }
}

impl core::error::Error for Error {}

mod entry {
  use super::{Key, Output, TombstoneValidator};
  use core::fmt::Debug;
  use crossbeam_skiplist::map;
  use dbutils::{equivalentor::Ascend, state::State};
  use snapshotor::{CursorExt, DoubleEndedCursorExt, Entry as _, NoopValidator};
  pub struct MapEntry<'a, K, V>(pub(super) map::Entry<'a, Key<K>, Option<V>>);
  impl<'a, K, V> From<map::Entry<'a, Key<K>, Option<V>>> for MapEntry<'a, K, V> {
    #[inline]
    fn from(src: map::Entry<'a, Key<K>, Option<V>>) -> Self {
      Self(src)
    }
  }
  impl<K, V> Clone for MapEntry<'_, K, V> {
    #[inline]
    fn clone(&self) -> Self {
      Self(self.0.clone())
    }
  }
  impl<K, V> snapshotor::Entry for MapEntry<'_, K, V> {
    type Key = K;
    type Value = Option<V>;
    type Version = u64;
    #[inline]
    fn key(&self) -> &Self::Key {
      &self.0.key().key
    }
    #[inline]
    fn value(&self) -> &Self::Value {
      self.0.value()
    }
    #[inline]
    fn version(&self) -> Self::Version {
      self.0.key().version
    }
  }
  impl<K, V> snapshotor::Cursor for MapEntry<'_, K, V>
  where
    K: Ord,
  {
    fn next(&self) -> Option<Self>
    where
      Self: Sized,
    {
      self.0.next().map(MapEntry)
    }
  }
  impl<K, V> snapshotor::DoubleEndedCursor for MapEntry<'_, K, V>
  where
    K: Ord,
  {
    fn next_back(&self) -> Option<Self>
    where
      Self: Sized,
    {
      self.0.prev().map(MapEntry)
    }
  }
  /// A reference-counted entry in a map.
  pub struct Entry<'a, K, V, S> {
    pub(super) ent: MapEntry<'a, K, V>,
    query_version: u64,
    _m: core::marker::PhantomData<S>,
  }
  impl<'a, K: Debug, V: Debug, S> Debug for Entry<'a, K, V, S>
  where
    S: Output<'a, V>,
    S::Output: Debug,
    K: Debug,
    V: Debug,
  {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("Entry")
        .field("version", &self.version())
        .field("key", &self.key())
        .field("value", &self.value())
        .finish()
    }
  }
  impl<K, V, S> Clone for Entry<'_, K, V, S> {
    #[inline]
    fn clone(&self) -> Self {
      Self {
        ent: self.ent.clone(),
        query_version: self.query_version,
        _m: core::marker::PhantomData,
      }
    }
  }
  impl<'a, K, V, S> Entry<'a, K, V, S> {
    /// Returns the version of the entry.
    #[inline]
    pub fn version(&self) -> u64 {
      self.ent.version()
    }
    /// Returns the key of the entry.
    #[inline]
    pub fn key(&self) -> &'a K {
      &self.ent.0.key().key
    }
    /// Returns the value of the entry.
    #[inline]
    pub fn value(&self) -> S::Output
    where
      S: Output<'a, V>,
    {
      S::output(self.ent.0.value().as_ref())
    }
    #[inline]
    pub(super) fn new(entry: MapEntry<'a, K, V>, query_version: u64) -> Self {
      Self {
        ent: entry,
        query_version,
        _m: core::marker::PhantomData,
      }
    }
  }
  impl<K, V, S> Entry<'_, K, V, S>
  where
    K: Ord,
    S: State,
  {
    /// Returns the next entry in the map.
    pub fn next(&self) -> Option<Self> {
      if !S::ALWAYS_VALID {
        let mut curr = self.ent.0.next();
        loop {
          match curr {
            None => return None,
            Some(ent) => {
              let curr_key = ent.key();
              if curr_key.version > self.query_version {
                curr = ent.next();
                continue;
              }
              break Some(MapEntry(ent));
            }
          }
        }
      } else {
        self.ent.next_dedup(
          &self.query_version,
          &Ascend,
          &NoopValidator,
          &TombstoneValidator,
        )
      }
      .map(|ent| Entry::new(ent, self.query_version))
    }
    /// Returns the previous entry in the map.
    pub fn prev(&self) -> Option<Self> {
      if !S::ALWAYS_VALID {
        let mut curr = self.ent.0.prev();
        loop {
          match curr {
            None => return None,
            Some(ent) => {
              let curr_key = ent.key();
              if curr_key.version > self.query_version {
                curr = ent.prev();
                continue;
              }
              break Some(MapEntry(ent));
            }
          }
        }
      } else {
        self.ent.next_back_dedup(
          &self.query_version,
          &Ascend,
          &NoopValidator,
          &TombstoneValidator,
        )
      }
      .map(|ent| Entry::new(ent, self.query_version))
    }
  }
}
pub use entry::Entry;
mod iter {
  use dbutils::{
    equivalentor::Ascend,
    state::{Active, MaybeTombstone, State},
  };
  use snapshotor::{dedup, valid, Builder, NoopValidator};

  use super::{entry::MapEntry, Entry, SkipMap, TombstoneValidator};
  /// The state of the iterator.
  pub trait IterState<K, V>: sealed::Sealed<K, V> {}
  impl<K, V, T> IterState<K, V> for T where T: sealed::Sealed<K, V> {}
  mod sealed {
    use super::*;
    pub trait Sealed<K, V>: State {
      type Iter<'a>;
    }
    impl<K, V> Sealed<K, V> for Active
    where
      K: 'static,
      V: 'static,
    {
      type Iter<'a> = dedup::Iter<
        MapEntry<'a, K, V>,
        Rewinder<'a, K, V>,
        Ascend,
        NoopValidator,
        TombstoneValidator,
      >;
    }
    impl<K, V> Sealed<K, V> for MaybeTombstone
    where
      K: 'static,
      V: 'static,
    {
      type Iter<'a> =
        valid::Iter<MapEntry<'a, K, V>, Rewinder<'a, K, V>, Ascend, NoopValidator, NoopValidator>;
    }
  }
  pub struct Rewinder<'a, K, V>(&'a SkipMap<K, V>);
  impl<'a, K, V> snapshotor::Rewindable for Rewinder<'a, K, V>
  where
    K: Ord + 'static,
    V: 'static,
  {
    type Entry = MapEntry<'a, K, V>;
    fn first(&self) -> Option<Self::Entry> {
      self.0.inner.front().map(MapEntry)
    }
    fn last(&self) -> Option<Self::Entry> {
      self.0.inner.back().map(MapEntry)
    }
  }
  /// a
  pub struct Iter<'a, K, V, S>
  where
    S: IterState<K, V>,
  {
    iter: S::Iter<'a>,
    query_version: u64,
  }
  impl<'a, K, V> Iter<'a, K, V, Active>
  where
    K: Ord + 'static,
    V: 'static,
  {
    #[inline]
    pub(super) fn new(version: u64, map: &'a super::SkipMap<K, V>) -> Self {
      Self {
        iter: Builder::new(Rewinder(map))
          .with_value_validator(TombstoneValidator)
          .iter(version),
        query_version: version,
      }
    }
  }
  impl<'a, K, V> Iter<'a, K, V, MaybeTombstone>
  where
    K: Ord + 'static,
    V: 'static,
  {
    #[inline]
    pub(super) fn with_tombstone(version: u64, map: &'a super::SkipMap<K, V>) -> Self {
      Self {
        iter: Builder::new(Rewinder(map)).iter(version),
        query_version: version,
      }
    }
  }
  impl<'a, K, V, S> Iterator for Iter<'a, K, V, S>
  where
    K: Ord + 'static,
    V: 'static,
    S: IterState<K, V>,
    S::Iter<'a>: Iterator<Item = MapEntry<'a, K, V>>,
  {
    type Item = Entry<'a, K, V, S>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
      self
        .iter
        .next()
        .map(|ent| Entry::new(ent, self.query_version))
    }
  }
  impl<'a, K, V, S> DoubleEndedIterator for Iter<'a, K, V, S>
  where
    K: Ord + 'static,
    V: 'static,
    S: IterState<K, V>,
    S::Iter<'a>: DoubleEndedIterator<Item = MapEntry<'a, K, V>>,
  {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
      self
        .iter
        .next_back()
        .map(|ent| Entry::new(ent, self.query_version))
    }
  }
}
pub use iter::Iter;
mod range {
  use super::{entry::MapEntry, Entry, Query, TombstoneValidator};
  use dbutils::{
    equivalent::Comparable,
    equivalentor::Ascend,
    state::{Active, MaybeTombstone, State},
  };
  use snapshotor::{dedup, valid, Builder, NoopValidator, Seekable};
  use std::ops::{Bound, RangeBounds};
  /// The state of the range.
  pub trait RangeState<K, V>: sealed::Sealed<K, V> {}
  impl<K, V, T> RangeState<K, V> for T where T: sealed::Sealed<K, V> {}
  mod sealed {
    use super::*;
    pub trait Sealed<K, V>: State {
      type Range<'a, Q, R>
      where
        K: Ord + Comparable<Q>,
        Q: ?Sized,
        R: RangeBounds<Q>;
    }
    impl<K, V> Sealed<K, V> for Active
    where
      K: 'static,
      V: 'static,
    {
      type Range<'a, Q, R>
        = dedup::Range<
        R,
        Q,
        Seeker<'a, K, V>,
        MapEntry<'a, K, V>,
        Ascend,
        NoopValidator,
        TombstoneValidator,
      >
      where
        K: Ord + Comparable<Q>,
        Q: ?Sized,
        R: RangeBounds<Q>;
    }
    impl<K, V> Sealed<K, V> for MaybeTombstone
    where
      K: 'static,
      V: 'static,
    {
      type Range<'a, Q, R>
        = valid::Range<
        R,
        Q,
        Seeker<'a, K, V>,
        MapEntry<'a, K, V>,
        Ascend,
        NoopValidator,
        NoopValidator,
      >
      where
        K: Ord + Comparable<Q>,
        Q: ?Sized,
        R: RangeBounds<Q>;
    }
  }
  pub struct Seeker<'a, K, V> {
    map: &'a super::SkipMap<K, V>,
    query_version: u64,
  }
  impl<'a, K, V, Q> Seekable<Q> for Seeker<'a, K, V>
  where
    K: Ord + Comparable<Q> + 'static,
    V: 'static,
    Q: ?Sized,
  {
    type Entry = MapEntry<'a, K, V>;
    fn lower_bound(&self, bound: Bound<&Q>) -> Option<Self::Entry> {
      self
        .map
        .inner
        .lower_bound(bound.map(|q| Query::new(self.query_version, q)).as_ref())
        .map(MapEntry)
    }
    fn upper_bound(&self, bound: Bound<&Q>) -> Option<Self::Entry> {
      self
        .map
        .inner
        .upper_bound(bound.map(|q| Query::new(0, q)).as_ref())
        .map(MapEntry)
    }
  }
  /// a
  pub struct Range<'a, K, V, S, Q, R>
  where
    K: Ord + Comparable<Q> + 'static,
    V: 'static,
    R: RangeBounds<Q>,
    Q: ?Sized,
    S: RangeState<K, V>,
  {
    iter: S::Range<'a, Q, R>,
    version: u64,
  }
  impl<'a, K, V, Q, R> Range<'a, K, V, Active, Q, R>
  where
    K: Ord + Comparable<Q> + 'static,
    V: 'static,
    R: RangeBounds<Q>,
    Q: ?Sized,
  {
    #[inline]
    pub(super) fn new(version: u64, map: &'a super::SkipMap<K, V>, range: R) -> Self {
      let seeker = Seeker {
        map,
        query_version: version,
      };
      Self {
        iter: Builder::new(seeker)
          .with_value_validator(TombstoneValidator)
          .range(version, range),
        version,
      }
    }
  }
  impl<'a, K, V, Q, R> Range<'a, K, V, MaybeTombstone, Q, R>
  where
    K: Ord + Comparable<Q> + 'static,
    V: 'static,
    R: RangeBounds<Q>,
    Q: ?Sized,
  {
    #[inline]
    pub(super) fn with_tombstone(version: u64, map: &'a super::SkipMap<K, V>, range: R) -> Self {
      let seeker = Seeker {
        map,
        query_version: version,
      };
      Self {
        iter: Builder::new(seeker).range(version, range),
        version,
      }
    }
  }
  impl<'a, K, V, S, Q, R> Iterator for Range<'a, K, V, S, Q, R>
  where
    K: Ord + Comparable<Q> + 'static,
    V: 'static,
    R: RangeBounds<Q>,
    Q: ?Sized,
    S: RangeState<K, V>,
    S::Range<'a, Q, R>: Iterator<Item = MapEntry<'a, K, V>>,
  {
    type Item = Entry<'a, K, V, S>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
      self.iter.next().map(|ent| Entry::new(ent, self.version))
    }
  }
  impl<'a, K, V, S, Q, R> DoubleEndedIterator for Range<'a, K, V, S, Q, R>
  where
    K: Ord + Comparable<Q> + 'static,
    V: 'static,
    Q: ?Sized,
    R: RangeBounds<Q>,
    S: RangeState<K, V>,
    S::Range<'a, Q, R>: DoubleEndedIterator<Item = MapEntry<'a, K, V>>,
  {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
      self
        .iter
        .next_back()
        .map(|ent| Entry::new(ent, self.version))
    }
  }
}
pub use range::Range;
struct Key<K> {
  key: K,
  version: u64,
}
impl<K> Key<K> {
  #[inline]
  const fn new(key: K, version: u64) -> Self {
    Self { key, version }
  }
}
impl<K> PartialEq for Key<K>
where
  K: PartialEq,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.key == other.key && self.version == other.version
  }
}
impl<K> Eq for Key<K> where K: Eq {}
impl<K> PartialOrd for Key<K>
where
  K: PartialOrd,
{
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
    self
      .key
      .partial_cmp(&other.key)
      .map(|o| o.then_with(|| other.version.cmp(&self.version)))
  }
}
impl<K> Ord for Key<K>
where
  K: Ord,
{
  #[inline]
  fn cmp(&self, other: &Self) -> cmp::Ordering {
    self
      .key
      .cmp(&other.key)
      .then_with(|| other.version.cmp(&self.version))
  }
}
struct Query<'a, Q: ?Sized, K: ?Sized> {
  _m: PhantomData<K>,
  query: &'a Q,
  version: u64,
}
impl<'a, Q: ?Sized, K: ?Sized> Query<'a, Q, K> {
  #[inline]
  fn new(version: u64, query: &'a Q) -> Self {
    Self {
      _m: PhantomData,
      query,
      version,
    }
  }
}
impl<Q, K> Equivalent<Query<'_, Q, K>> for Key<K>
where
  K: Equivalent<Q>,
  Q: ?Sized,
{
  #[inline]
  fn equivalent(&self, key: &Query<'_, Q, K>) -> bool {
    Equivalent::equivalent(&self.key, key.query) && key.version == self.version
  }
}
impl<Q, K> Comparable<Query<'_, Q, K>> for Key<K>
where
  K: Comparable<Q>,
  Q: ?Sized,
{
  #[inline]
  fn compare(&self, key: &Query<'_, Q, K>) -> cmp::Ordering {
    Comparable::compare(&self.key, key.query).then_with(|| key.version.cmp(&self.version))
  }
}

pub struct SkipMap<K, V> {
  inner: CSkipMap<Key<K>, Option<V>>,
  min_version: AtomicU64,
  max_version: AtomicU64,
  last_discard_version: AtomicU64,
}
impl<K, V> Default for SkipMap<K, V> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}
impl<K, V> SkipMap<K, V> {
  pub fn new() -> Self {
    Self {
      inner: CSkipMap::new(),
      min_version: AtomicU64::new(u64::MAX),
      max_version: AtomicU64::new(0),
      last_discard_version: AtomicU64::new(0),
    }
  }

  #[inline]
  pub fn may_contain_version(&self, version: u64) -> bool {
    version >= self.min_version.load(Ordering::Acquire)
  }

  #[inline]
  pub fn minimum_version(&self) -> u64 {
    self.min_version.load(Ordering::Acquire)
  }

  #[inline]
  pub fn maximum_version(&self) -> u64 {
    self.max_version.load(Ordering::Acquire)
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.inner.len()
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }
  fn update_versions(&self, version: u64) {
    let _ = self
      .min_version
      .fetch_update(Ordering::Release, Ordering::Acquire, |min_version| {
        min_version.gt(&version).then_some(version)
      });
    let _ = self
      .max_version
      .fetch_update(Ordering::Release, Ordering::Acquire, |max_version| {
        max_version.lt(&version).then_some(version)
      });
  }
}
impl<K, V> SkipMap<K, V>
where
  K: Ord + 'static,
  V: 'static,
{
  #[inline]
  pub fn contains_key<Q>(&self, version: u64, key: &Q) -> bool
  where
    K: Comparable<Q>,
    Q: ?Sized,
  {
    if !self.may_contain_version(version) {
      return false;
    }
    match self
      .inner
      .lower_bound(Bound::Included(&Query::new(version, key)))
    {
      Some(entry) => {
        let k = entry.key();
        if !k.key.equivalent(key) {
          return false;
        }
        if entry.value().is_none() {
          return false;
        }
        true
      }
      None => false,
    }
  }

  #[inline]
  pub fn contains_key_with_tombstone<Q>(&self, version: u64, key: &Q) -> bool
  where
    K: Comparable<Q>,
    Q: ?Sized,
  {
    if !self.may_contain_version(version) {
      return false;
    }
    match self
      .inner
      .lower_bound(Bound::Included(&Query::new(version, key)))
    {
      Some(entry) => {
        let k = entry.key();
        if !k.key.equivalent(key) {
          return false;
        }
        true
      }
      None => false,
    }
  }

  pub fn get<Q>(&self, version: u64, key: &Q) -> Option<Entry<'_, K, V, Active>>
  where
    K: Comparable<Q>,
    Q: ?Sized,
  {
    if !self.may_contain_version(version) {
      return None;
    }
    match self
      .inner
      .lower_bound(Bound::Included(&Query::new(version, key)))
    {
      Some(entry) => {
        let k = entry.key();
        if !k.key.equivalent(key) {
          return None;
        }
        if entry.value().is_none() {
          return None;
        }
        Some(Entry::new(entry.into(), version))
      }
      None => None,
    }
  }

  pub fn get_with_tombstone<Q>(
    &self,
    version: u64,
    key: &Q,
  ) -> Option<Entry<'_, K, V, MaybeTombstone>>
  where
    K: Comparable<Q>,
    Q: ?Sized,
  {
    if !self.may_contain_version(version) {
      return None;
    }
    match self
      .inner
      .lower_bound(Bound::Included(&Query::new(version, key)))
    {
      Some(entry) => {
        let k = entry.key();
        if !k.key.equivalent(key) {
          return None;
        }
        Some(Entry::new(entry.into(), version))
      }
      None => None,
    }
  }

  pub fn lower_bound<'a, Q>(
    &'a self,
    version: u64,
    bound: Bound<&'a Q>,
  ) -> Option<Entry<'a, K, V, Active>>
  where
    K: Comparable<Q> + 'static,
    V: 'static,
    Q: ?Sized,
  {
    if !self.may_contain_version(version) {
      return None;
    }
    self.range(version, (bound, Bound::Unbounded)).next()
  }

  pub fn lower_bound_with_tombstone<'a, Q>(
    &'a self,
    version: u64,
    bound: Bound<&Q>,
  ) -> Option<Entry<'a, K, V, MaybeTombstone>>
  where
    K: Comparable<Q>,
    Q: ?Sized,
  {
    if !self.may_contain_version(version) {
      return None;
    }
    self.range_all(version, (bound, Bound::Unbounded)).next()
  }

  pub fn upper_bound<'a, Q>(
    &'a self,
    version: u64,
    bound: Bound<&Q>,
  ) -> Option<Entry<'a, K, V, Active>>
  where
    K: Comparable<Q> + 'static,
    V: 'static,
    Q: ?Sized,
  {
    if !self.may_contain_version(version) {
      return None;
    }
    self.range(version, (Bound::Unbounded, bound)).next_back()
  }

  pub fn upper_bound_with_tombstone<'a, Q>(
    &'a self,
    version: u64,
    bound: Bound<&Q>,
  ) -> Option<Entry<'a, K, V, MaybeTombstone>>
  where
    K: Comparable<Q> + core::fmt::Debug,
    Q: ?Sized,
  {
    if !self.may_contain_version(version) {
      return None;
    }
    self
      .range_all(version, (Bound::Unbounded, bound))
      .next_back()
  }

  pub fn front(&self, version: u64) -> Option<Entry<'_, K, V, Active>>
  where
    K: 'static,
    V: 'static,
  {
    if !self.may_contain_version(version) {
      return None;
    }
    self.iter(version).next()
  }

  pub fn front_with_tombstone(&self, version: u64) -> Option<Entry<'_, K, V, MaybeTombstone>> {
    if !self.may_contain_version(version) {
      return None;
    }
    self.iter_all(version).next()
  }

  pub fn back(&self, version: u64) -> Option<Entry<'_, K, V, Active>>
  where
    K: 'static,
    V: 'static,
  {
    if !self.may_contain_version(version) {
      return None;
    }
    self.iter(version).next_back()
  }

  pub fn back_with_tombstone(&self, version: u64) -> Option<Entry<'_, K, V, MaybeTombstone>> {
    if !self.may_contain_version(version) {
      return None;
    }
    self.iter_all(version).next_back()
  }

  pub fn iter(&self, version: u64) -> Iter<'_, K, V, Active> {
    Iter::new(version, self)
  }
  pub fn iter_all(&self, version: u64) -> Iter<'_, K, V, MaybeTombstone> {
    Iter::with_tombstone(version, self)
  }

  pub fn range<Q, R>(&self, version: u64, range: R) -> Range<'_, K, V, Active, Q, R>
  where
    R: RangeBounds<Q>,
    K: Comparable<Q>,
    Q: ?Sized,
  {
    Range::new(version, self, range)
  }
  pub fn range_all<Q, R>(&self, version: u64, range: R) -> Range<'_, K, V, MaybeTombstone, Q, R>
  where
    R: RangeBounds<Q>,
    K: Comparable<Q>,
    Q: ?Sized,
  {
    Range::with_tombstone(version, self, range)
  }
}
impl<K, V> SkipMap<K, V>
where
  K: Ord + Send + 'static,
  V: Send + 'static,
{
  pub fn insert(&self, version: u64, key: K, value: V) -> Result<Entry<'_, K, V, Active>, Error> {
    self
      .check_discard(version)
      .map(|_| self.insert_in(version, key, value))
  }

  pub fn insert_unchecked(&self, version: u64, key: K, value: V) -> Entry<'_, K, V, Active> {
    self
      .check_discard(version)
      .expect("version has already been discarded");
    self.insert_in(version, key, value)
  }

  pub fn compare_insert<F>(
    &self,
    version: u64,
    key: K,
    value: V,
    compare_fn: F,
  ) -> Result<Entry<'_, K, V, Active>, Error>
  where
    F: Fn(Option<&V>) -> bool,
  {
    self
      .check_discard(version)
      .map(|_| self.compare_insert_in(version, key, value, compare_fn))
  }

  pub fn compare_insert_unchecked<F>(
    &self,
    version: u64,
    key: K,
    value: V,
    compare_fn: F,
  ) -> Entry<'_, K, V, Active>
  where
    F: Fn(Option<&V>) -> bool,
  {
    self
      .check_discard(version)
      .expect("version has already been discarded");
    self.compare_insert_in(version, key, value, compare_fn)
  }

  pub fn remove(&self, version: u64, key: K) -> Result<Option<Entry<'_, K, V, Active>>, Error> {
    self
      .check_discard(version)
      .map(|_| self.remove_in(version, key))
  }

  pub fn remove_unchecked(&self, version: u64, key: K) -> Option<Entry<'_, K, V, Active>> {
    self
      .check_discard(version)
      .expect("version has already been discarded");
    self.remove_in(version, key)
  }
  #[inline]
  fn check_discard(&self, version: u64) -> Result<(), Error> {
    let last = self.last_discard_version.load(Ordering::Acquire);
    if last != 0 && last >= version {
      return Err(Error::AlreadyDiscarded(version));
    }
    Ok(())
  }
  fn insert_in(&self, version: u64, key: K, value: V) -> Entry<'_, K, V, Active> {
    let ent = self.inner.insert(Key::new(key, version), Some(value));
    self.update_versions(version);
    Entry::new(ent.into(), version)
  }
  fn compare_insert_in(
    &self,
    version: u64,
    key: K,
    value: V,
    compare_fn: impl Fn(Option<&V>) -> bool,
  ) -> Entry<'_, K, V, Active> {
    let ent = self
      .inner
      .compare_insert(Key::new(key, version), Some(value), |old_value| {
        compare_fn(old_value.as_ref())
      });
    self.update_versions(version);
    Entry::new(ent.into(), version)
  }
  #[inline]
  fn remove_in(&self, version: u64, key: K) -> Option<Entry<'_, K, V, Active>> {
    let ent = self.inner.insert(Key::new(key, version), None);
    self.update_versions(version);
    let next = ent.next()?;
    if next.key().key.eq(&ent.key().key) && next.value().is_some() {
      return Some(Entry::new(next.into(), version));
    }
    None
  }

  pub fn compact(&self, version: u64) -> u64
  where
    V: Sync,
  {
    match self
      .last_discard_version
      .fetch_update(Ordering::SeqCst, Ordering::Acquire, |val| {
        if val >= version {
          None
        } else {
          Some(version)
        }
      }) {
      Ok(_) => {}
      Err(version) => return version,
    }
    let min_version = self.min_version.load(Ordering::Acquire);
    for ent in self.inner.iter() {
      if ent.key().version <= version {
        ent.remove();
      }
    }
    let _ =
      self
        .min_version
        .compare_exchange(min_version, version, Ordering::AcqRel, Ordering::Relaxed);
    version
  }
}

pub struct TombstoneValidator;

impl<V> snapshotor::Validator<Option<V>> for TombstoneValidator {
  #[inline]
  fn validate(&self, value: &Option<V>) -> bool {
    value.is_some()
  }
}

pub trait Output<'a, V: 'a> {
  type Output: 'a;

  fn output(data: Option<&'a V>) -> Self::Output;
}

impl<'a, V: 'a> Output<'a, V> for dbutils::state::Active {
  type Output = &'a V;

  #[inline]
  fn output(data: Option<&'a V>) -> Self::Output {
    data.expect("entry in Active state must have a value")
  }
}

impl<'a, V: 'a> Output<'a, V> for dbutils::state::MaybeTombstone {
  type Output = Option<&'a V>;

  #[inline]
  fn output(data: Option<&'a V>) -> Self::Output {
    data
  }
}

#[test]
fn basic() {
  let map = SkipMap::new();
  map.insert(0, "key1", 1).unwrap();
  map.insert(0, "key3", 3).unwrap();
  map.insert(0, "key2", 2).unwrap();

  {
    let it = map.iter_all(0);
    for (idx, ent) in it.enumerate() {
      assert_eq!(ent.version(), 0);
      assert_eq!(ent.key(), &format!("key{}", idx + 1));
      assert_eq!(ent.value().unwrap(), &(idx + 1));
    }
  }

  map.insert_unchecked(1, "a", 1);
  map.insert_unchecked(2, "a", 2);

  {
    let mut it = map.iter_all(2);
    let ent = it.next().unwrap();
    assert_eq!(ent.version(), 2);
    assert_eq!(ent.key(), &"a");
    assert_eq!(ent.value().unwrap(), &2);

    let ent = it.next().unwrap();
    assert_eq!(ent.version(), 1);
    assert_eq!(ent.key(), &"a");
    assert_eq!(ent.value().unwrap(), &1);
  }

  map.insert_unchecked(2, "b", 2);
  map.insert_unchecked(1, "b", 1);

  {
    let mut it = map.range_all(2, "b"..);

    let ent = it.next().unwrap();
    assert_eq!(ent.version(), 2);
    assert_eq!(ent.key(), &"b");
    assert_eq!(ent.value().unwrap(), &2);

    let ent = it.next().unwrap();
    assert_eq!(ent.version(), 1);
    assert_eq!(ent.key(), &"b");
    assert_eq!(ent.value().unwrap(), &1);
  }
}

#[test]
fn iter_all_mvcc() {
  let map = SkipMap::new();
  map.insert_unchecked(1, "a", "a1");
  map.insert_unchecked(3, "a", "a2");
  map.insert_unchecked(1, "c", "c1");
  map.insert_unchecked(3, "c", "c2");

  let mut it = map.iter_all(0);
  let mut num = 0;
  while it.next().is_some() {
    num += 1;
  }

  assert_eq!(num, 0);

  let mut it = map.iter_all(1);
  let a1 = it.next().unwrap();
  assert_eq!(a1.version(), 1);
  assert_eq!(a1.key(), &"a");
  assert_eq!(a1.value().unwrap(), &"a1");

  let c1 = it.next().unwrap();
  assert_eq!(c1.version(), 1);
  assert_eq!(c1.key(), &"c");
  assert_eq!(c1.value().unwrap(), &"c1");

  let mut it = map.iter_all(2);
  let a1 = it.next().unwrap();
  assert_eq!(a1.version(), 1);
  assert_eq!(a1.key(), &"a");
  assert_eq!(a1.value().unwrap(), &"a1");

  let c1 = it.next().unwrap();
  assert_eq!(c1.version(), 1);
  assert_eq!(c1.key(), &"c");
  assert_eq!(c1.value().unwrap(), &"c1");

  let mut it = map.iter_all(3);
  let a2 = it.next().unwrap();
  assert_eq!(a2.version(), 3);
  assert_eq!(a2.key(), &"a");
  assert_eq!(a2.value().unwrap(), &"a2");

  let a1 = it.next().unwrap();
  assert_eq!(a1.version(), 1);
  assert_eq!(a1.key(), &"a");
  assert_eq!(a1.value().unwrap(), &"a1");

  let c2 = it.next().unwrap();
  assert_eq!(c2.version(), 3);
  assert_eq!(c2.key(), &"c");
  assert_eq!(c2.value().unwrap(), &"c2");

  let c1 = it.next().unwrap();
  assert_eq!(c1.version(), 1);
  assert_eq!(c1.key(), &"c");
  assert_eq!(c1.value().unwrap(), &"c1");
}

#[test]
fn get_mvcc() {
  let map = SkipMap::new();
  map.insert_unchecked(1, "a", "a1");
  map.insert_unchecked(3, "a", "a2");
  map.insert_unchecked(1, "c", "c1");
  map.insert_unchecked(3, "c", "c2");

  let ent = map.get(1, "a").unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a1");

  let ent = map.get(2, "a").unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a1");

  let ent = map.get(3, "a").unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a2");

  let ent = map.get(4, "a").unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a2");

  assert!(map.get(0, "b").is_none());
  assert!(map.get(1, "b").is_none());
  assert!(map.get(2, "b").is_none());
  assert!(map.get(3, "b").is_none());
  assert!(map.get(4, "b").is_none());

  let ent = map.get(1, "c").unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.get(2, "c").unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.get(3, "c").unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");

  let ent = map.get(4, "c").unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");

  assert!(map.get(5, "d").is_none());
}

#[test]
fn gt() {
  let map = SkipMap::new();
  map.insert_unchecked(1, "a", "a1");
  map.insert_unchecked(3, "a", "a2");
  map.insert_unchecked(1, "c", "c1");
  map.insert_unchecked(3, "c", "c2");
  map.insert_unchecked(5, "c", "c3");

  let ent = map.lower_bound(1, Bound::Excluded("")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a1");

  let ent = map.lower_bound(2, Bound::Excluded("")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a1");

  let ent = map.lower_bound(3, Bound::Excluded("")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a2");

  let ent = map.lower_bound(1, Bound::Excluded("a")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.lower_bound(2, Bound::Excluded("a")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.lower_bound(3, Bound::Excluded("a")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");

  let ent = map.lower_bound(1, Bound::Excluded("b")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.lower_bound(2, Bound::Excluded("b")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.lower_bound(3, Bound::Excluded("b")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");

  let ent = map.lower_bound(4, Bound::Excluded("b")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");

  let ent = map.lower_bound(5, Bound::Excluded("b")).unwrap();
  assert_eq!(ent.version(), 5);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c3");

  let ent = map.lower_bound(6, Bound::Excluded("b")).unwrap();
  assert_eq!(ent.version(), 5);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c3");

  assert!(map.lower_bound(1, Bound::Excluded("c")).is_none());
  assert!(map.lower_bound(2, Bound::Excluded("c")).is_none());
  assert!(map.lower_bound(3, Bound::Excluded("c")).is_none());
  assert!(map.lower_bound(4, Bound::Excluded("c")).is_none());
  assert!(map.lower_bound(5, Bound::Excluded("c")).is_none());
  assert!(map.lower_bound(6, Bound::Excluded("c")).is_none());
}

#[test]
fn ge() {
  let map = SkipMap::new();
  map.insert_unchecked(1, "a", "a1");
  map.insert_unchecked(3, "a", "a2");
  map.insert_unchecked(1, "c", "c1");
  map.insert_unchecked(3, "c", "c2");

  assert!(map.lower_bound(0, Bound::Included("a")).is_none());
  assert!(map.lower_bound(0, Bound::Included("b")).is_none());
  assert!(map.lower_bound(0, Bound::Included("c")).is_none());

  let ent = map.lower_bound(1, Bound::Included("a")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a1");

  let ent = map.lower_bound(2, Bound::Included("a")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a1");

  let ent = map.lower_bound(3, Bound::Included("a")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a2");

  let ent = map.lower_bound(4, Bound::Included("a")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a2");

  let ent = map.lower_bound(1, Bound::Included("b")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.lower_bound(2, Bound::Included("b")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.lower_bound(3, Bound::Included("b")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");

  let ent = map.lower_bound(4, Bound::Included("b")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");

  let ent = map.lower_bound(1, Bound::Included("c")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.lower_bound(2, Bound::Included("c")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.lower_bound(3, Bound::Included("c")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");

  let ent = map.lower_bound(4, Bound::Included("c")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");

  assert!(map.lower_bound(0, Bound::Included("d")).is_none());
  assert!(map.lower_bound(1, Bound::Included("d")).is_none());
  assert!(map.lower_bound(2, Bound::Included("d")).is_none());
  assert!(map.lower_bound(3, Bound::Included("d")).is_none());
  assert!(map.lower_bound(4, Bound::Included("d")).is_none());
}

#[test]
fn le() {
  let map = SkipMap::new();
  map.insert_unchecked(1, "a", "a1");
  map.insert_unchecked(3, "a", "a2");
  map.insert_unchecked(1, "c", "c1");
  map.insert_unchecked(3, "c", "c2");

  assert!(map.upper_bound(0, Bound::Included("a")).is_none());
  assert!(map.upper_bound(0, Bound::Included("b")).is_none());
  assert!(map.upper_bound(0, Bound::Included("c")).is_none());

  let ent = map.upper_bound(1, Bound::Included("a")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a1");

  let ent = map.upper_bound(2, Bound::Included("a")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a1");

  let ent = map.upper_bound(3, Bound::Included("a")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a2");

  let ent = map.upper_bound(4, Bound::Included("a")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a2");

  let ent = map.upper_bound(1, Bound::Included("b")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a1");

  let ent = map.upper_bound(2, Bound::Included("b")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a1");

  let ent = map.upper_bound(3, Bound::Included("b")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a2");

  let ent = map.upper_bound(4, Bound::Included("b")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a2");

  let ent = map.upper_bound(1, Bound::Included("c")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.upper_bound(2, Bound::Included("c")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.upper_bound(3, Bound::Included("c")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");

  let ent = map.upper_bound(4, Bound::Included("c")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");

  let ent = map.upper_bound(1, Bound::Included("d")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.upper_bound(2, Bound::Included("d")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.upper_bound(3, Bound::Included("d")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");

  let ent = map.upper_bound(4, Bound::Included("d")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");
}

#[test]
fn lt() {
  let map = SkipMap::new();

  map.insert_unchecked(1, "a", "a1");
  map.insert_unchecked(3, "a", "a2");
  map.insert_unchecked(1, "c", "c1");
  map.insert_unchecked(3, "c", "c2");

  assert!(map.upper_bound(0, Bound::Excluded("a")).is_none());
  assert!(map.upper_bound(0, Bound::Excluded("b")).is_none());
  assert!(map.upper_bound(0, Bound::Excluded("c")).is_none());
  assert!(map.upper_bound(1, Bound::Excluded("a")).is_none());
  assert!(map.upper_bound(2, Bound::Excluded("a")).is_none());

  let ent = map.upper_bound(1, Bound::Excluded("b")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a1");

  let ent = map.upper_bound(2, Bound::Excluded("b")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a1");

  let ent = map.upper_bound(3, Bound::Excluded("b")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a2");

  let ent = map.upper_bound(4, Bound::Excluded("b")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a2");

  let ent = map.upper_bound(1, Bound::Excluded("c")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a1");

  let ent = map.upper_bound(2, Bound::Excluded("c")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a1");

  let ent = map.upper_bound(3, Bound::Excluded("c")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a2");

  let ent = map.upper_bound(4, Bound::Excluded("c")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"a");
  assert_eq!(ent.value(), &"a2");

  let ent = map.upper_bound(1, Bound::Excluded("d")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.upper_bound(2, Bound::Excluded("d")).unwrap();
  assert_eq!(ent.version(), 1);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c1");

  let ent = map.upper_bound(3, Bound::Excluded("d")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");

  let ent = map.upper_bound(4, Bound::Excluded("d")).unwrap();
  assert_eq!(ent.version(), 3);
  assert_eq!(ent.key(), &"c");
  assert_eq!(ent.value(), &"c2");
}

#[test]
fn all_versions_iter_forwards() {
  const N: usize = 100;

  let map = SkipMap::new();
  for i in 0..N {
    map.insert_unchecked(0, i, i);
    map.remove(1, i).unwrap();
  }

  let it = map.iter_all(0);
  let mut i = 0;
  for entry in it {
    assert_eq!(entry.version(), 0);
    assert_eq!(entry.key(), &i);
    assert_eq!(entry.value().unwrap(), &i);
    i += 1;
  }

  assert_eq!(i, N);

  let it = map.iter_all(1);

  let mut i = 0;
  for entry in it {
    if i % 2 == 1 {
      assert_eq!(entry.version(), 0);
      assert_eq!(*entry.key(), i / 2);
      assert_eq!(entry.value().unwrap(), &(i / 2));
    } else {
      assert_eq!(entry.version(), 1);
      assert_eq!(*entry.key(), i / 2);
      assert!(entry.value().is_none());
    }

    i += 1;
  }

  assert_eq!(i, 2 * N);

  let mut it = map.iter(1);
  assert!(it.next().is_none());
}

#[test]
fn all_versions_iter_backwards() {
  const N: usize = 100;

  let map = SkipMap::new();
  for i in 0..N {
    map.insert_unchecked(0, i, i);
    map.remove_unchecked(1, i);
  }

  let it = map.iter_all(0).rev();
  let mut i = 0;
  for entry in it {
    i += 1;
    assert_eq!(*entry.key(), N - i);
    assert_eq!(*entry.value().unwrap(), N - i);
  }
  assert_eq!(i, N);

  let it = map.iter_all(1).rev();
  let mut i = 0;
  for ref entry in it {
    if i % 2 == 0 {
      assert_eq!(entry.version(), 0);
      assert_eq!(*entry.key(), N - 1 - i / 2);
      assert_eq!(*entry.value().unwrap(), N - 1 - i / 2);
    } else {
      assert_eq!(entry.version(), 1);
      assert_eq!(*entry.key(), N - 1 - i / 2);
      assert!(entry.value().is_none());
    }
    i += 1;
  }
  assert_eq!(i, 2 * N);

  let mut it = map.iter(1);
  assert!(it.next().is_none());
}

#[test]
fn cursor_forwards() {
  const N: usize = 100;

  let map = SkipMap::new();
  for i in 0..N {
    map.insert_unchecked(0, i, i);
    map.remove_unchecked(1, i);
  }

  let mut ent = map.front(0);
  let mut i = 0;
  while let Some(entry) = ent {
    assert_eq!(entry.key(), &i);
    assert_eq!(entry.value(), &i);
    i += 1;
    ent = entry.next();
  }
  assert_eq!(i, N);

  let mut ent = map.front_with_tombstone(1);
  let mut i = 0;

  while let Some(ref entry) = ent {
    if i % 2 == 1 {
      assert_eq!(entry.version(), 0);
      assert_eq!(*entry.key(), i / 2);
      assert_eq!(*entry.value().unwrap(), i / 2);
    } else {
      assert_eq!(entry.version(), 1);
      assert_eq!(*entry.key(), i / 2);
      assert!(entry.value().is_none(), "{:?}", entry.value());
    }

    ent = entry.next();
    i += 1;
  }
  assert_eq!(i, 2 * N);
  let ent = map.front(1);
  assert!(ent.is_none());
}

#[test]
fn cursor_backwards() {
  const N: usize = 100;

  let map = SkipMap::new();
  for i in 0..N {
    map.insert_unchecked(0, i, i);
    map.remove_unchecked(1, i);
  }

  let mut ent = map.back(0);
  let mut i = 0;
  while let Some(entry) = ent {
    i += 1;
    assert_eq!(*entry.key(), N - i);
    assert_eq!(*entry.value(), N - i);
    ent = entry.prev();
  }
  assert_eq!(i, N);

  let mut ent = map.back_with_tombstone(1);
  let mut i = 0;
  while let Some(ref entry) = ent {
    if i % 2 == 0 {
      assert_eq!(entry.version(), 0);
      assert_eq!(*entry.key(), N - 1 - i / 2);
      assert_eq!(*entry.value().unwrap(), N - 1 - i / 2);
    } else {
      assert_eq!(entry.version(), 1);
      assert_eq!(*entry.key(), N - 1 - i / 2);
      assert!(entry.value().is_none());
    }
    i += 1;
    ent = entry.prev();
  }
  assert_eq!(i, 2 * N);
  let ent = map.back(1);
  assert!(ent.is_none());
}

#[test]
fn range_forwards() {
  const N: usize = 100;

  let map = SkipMap::new();
  for i in 0..N {
    map.insert_unchecked(0, i, i);
    map.remove_unchecked(1, i);
  }

  let it = map.range(0, ..=50);
  let mut i = 0;
  for entry in it {
    assert_eq!(entry.key(), &i);
    assert_eq!(entry.value(), &i);
    i += 1;
  }

  assert_eq!(i, 51);

  let it = map.range_all(1, ..=50);
  let mut i = 0;

  for entry in it {
    if i % 2 == 1 {
      assert_eq!(entry.version(), 0);
      assert_eq!(*entry.key(), i / 2);
      assert_eq!(*entry.value().unwrap(), i / 2);
    } else {
      assert_eq!(entry.version(), 1);
      assert_eq!(*entry.key(), i / 2);
      assert!(entry.value().is_none());
    }

    i += 1;
  }

  assert_eq!(i, 102);

  let mut it = map.range(1, ..=50);
  assert!(it.next().is_none());
}

#[test]
fn range_backwards() {
  const N: usize = 100;

  let map = SkipMap::new();
  for i in 0..N {
    map.insert_unchecked(0, i, i);
    map.remove_unchecked(1, i);
  }

  let it = map.range(0, ..=50).rev();
  let mut i = 0;
  for entry in it {
    i += 1;
    assert_eq!(*entry.key(), 51 - i);
    assert_eq!(*entry.value(), 51 - i);
  }
  assert_eq!(i, 51);

  let it = map.range_all(1, ..=50).rev();
  let mut i = 0;
  for ref entry in it {
    if i % 2 == 0 {
      assert_eq!(entry.version(), 0);
      assert_eq!(*entry.key(), 51 - 1 - i / 2);
      assert_eq!(*entry.value().unwrap(), 51 - 1 - i / 2);
    } else {
      assert_eq!(entry.version(), 1);
      assert_eq!(*entry.key(), 51 - 1 - i / 2);
      assert!(entry.value().is_none());
    }
    i += 1;
  }
  assert_eq!(i, 102);

  let mut it = map.range(1, ..=50);
  assert!(it.next().is_none());
}

#[test]
fn iter_latest() {
  const N: usize = 100;

  let map = SkipMap::new();
  for i in 0..N {
    map.insert_unchecked(0, i, i);
  }

  for i in 50..N {
    map.insert_unchecked(1, i, i + 1000);
  }

  for i in 0..50 {
    map.insert_unchecked(2, i, i + 1000);
  }

  let mut it = map.iter(4);
  let mut num = 0;

  for i in 0..N {
    let ent = it.next().unwrap();
    if i < 50 {
      assert_eq!(ent.version(), 2);
    } else {
      assert_eq!(ent.version(), 1);
    }
    assert_eq!(ent.key(), &i);
    assert_eq!(ent.value(), &(i + 1000));
    num += 1;
  }

  assert_eq!(num, N);
}

#[test]
fn range_latest() {
  const N: usize = 100;

  let map = SkipMap::new();
  for i in 0..N {
    map.insert_unchecked(0, i, i);
  }

  for i in 50..N {
    map.insert_unchecked(1, i, i + 1000);
  }

  for i in 0..50 {
    map.insert_unchecked(2, i, i + 1000);
  }

  let mut it = map.range::<usize, _>(4, ..);

  let mut num = 0;
  for i in 0..N {
    let ent = it.next().unwrap();
    if i < 50 {
      assert_eq!(ent.version(), 2);
    } else {
      assert_eq!(ent.version(), 1);
    }
    assert_eq!(ent.key(), &i);
    assert_eq!(ent.value(), &(i + 1000));
    num += 1;
  }

  assert_eq!(num, N);
}

#[test]
fn compact() {
  let map = SkipMap::new();

  map.insert_unchecked(1, "a", "a1");
  map.insert_unchecked(3, "a", "a2");
  map.insert_unchecked(1, "c", "c1");
  map.insert_unchecked(3, "c", "c2");

  let version = map.compact(2);
  assert_eq!(version, 2);

  for ent in map.iter_all(3) {
    assert_eq!(ent.version(), 3);
  }
}
