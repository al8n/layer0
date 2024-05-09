use super::*;

use core::{cell::RefCell, ops::{Bound, RangeBounds}};
use std::io;

use skl::{map::EntryRef as MapEntryRef, SkipMap};

pub use skl::{Ascend, Comparator, Descend, OccupiedValue};

pub use either::Either;

const EXTENSION: &str = "skl";

/// The options for creating a log.
#[viewit::viewit(getters(style = "move"), setters(prefix = "with"))]
pub struct CreateOptions {
  /// The file ID of the log.
  #[viewit(
    getter(const, attrs(doc = "Returns the file ID of the log.")),
    setter(attrs(doc = "Sets the file ID of the log."))
  )]
  fid: u32,

  /// The maximum size of the log.
  ///
  /// The log is backed by a mmaped file with the given size.
  /// So this size determines the mmaped file size.
  #[viewit(
    getter(const, attrs(doc = "Returns the size of the log.")),
    setter(attrs(doc = "Sets the size of the log."))
  )]
  size: usize,

  /// Whether to lock the log.
  ///
  /// If `true`, the log will be locked exlusively when it is created.
  #[viewit(
    getter(const, attrs(doc = "Returns if we should lock the log.")),
    setter(attrs(doc = "Sets whether to lock the log."))
  )]
  lock: bool,

  /// Whether to sync on write.
  /// 
  /// If `true`, the log will sync the data to disk on write.
  #[viewit(
    getter(const, attrs(doc = "Returns if we should sync on write.")),
    setter(attrs(doc = "Sets whether to sync on write."))
  )]
  sync_on_write: bool,
}

/// The options for opening a log.
#[viewit::viewit(getters(style = "move"), setters(prefix = "with"))]
pub struct OpenOptions {
  /// The file ID of the log.
  #[viewit(
    getter(const, attrs(doc = "Returns the file ID of the log.")),
    setter(attrs(doc = "Sets the file ID of the log."))
  )]
  fid: u32,

  /// Whether to lock the log.
  ///
  /// If `true`, the log will be locked exlusively when it is created.
  #[viewit(
    getter(const, attrs(doc = "Returns if we should lock the log.")),
    setter(attrs(doc = "Sets whether to lock the log."))
  )]
  lock: bool,
}

std::thread_local! {
  static BUF: RefCell<std::string::String> = RefCell::new(std::string::String::with_capacity(9));
}

/// Errors that can occur when working with a log.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// An I/O error occurred.
  #[error(transparent)]
  IO(#[from] io::Error),
  /// A log error occurred.
  #[error(transparent)]
  Log(#[from] skl::map::Error),
  /// Returned when writing the batch failed.
  #[error("failed to write batch at index {idx}: {source}")]
  WriteBatch {
    /// The index of the key-value pair that caused the error.
    idx: usize,
    /// The error that caused the failure.
    #[source]
    source: skl::map::Error,
  },
}

/// A append-only log based on on-disk [`SkipMap`] for key-value databases based on bitcask model.
pub struct BitcaskLog<C = Ascend> {
  map: SkipMap<Meta, C>,
  fid: u32,
  sync_on_write: bool,
  ro: bool,
}

impl<C> BitcaskLog<C> {
  /// Flushes outstanding memory map modifications to disk.
  #[inline]
  pub fn flush(&self) -> io::Result<()> {
    self.map.flush()
  }

  /// Asynchronously flushes outstanding memory map modifications to disk.
  #[inline]
  pub fn flush_async(&self) -> io::Result<()> {
    self.map.flush_async()
  }

  /// Returns the file ID of the log.
  #[inline]
  pub const fn fid(&self) -> u32 {
    self.fid
  }

  /// Returns `true` if the log is read only.
  #[inline]
  pub const fn read_only(&self) -> bool {
    self.ro
  }

  /// Returns the current size of the log.
  #[inline]
  pub fn size(&self) -> usize {
    self.map.size()
  }

  /// Returns the capacity of the log.
  #[inline]
  pub fn capacity(&self) -> usize {
    self.map.capacity()
  }
}

impl<C: Comparator> BitcaskLog<C> {
  /// Create a new log with the given options.
  pub fn create(cmp: C, opts: CreateOptions) -> io::Result<Self> {
    use std::fmt::Write;

    BUF.with(|buf| {
      let mut buf = buf.borrow_mut();
      buf.clear();
      write!(buf, "{:05}.{}", opts.fid, EXTENSION).unwrap();
      SkipMap::<Meta, C>::mmap_mut_with_comparator(buf.as_str(), opts.size, opts.lock, cmp)
        .map(|map| Self { map, fid: opts.fid, sync_on_write: opts.sync_on_write, ro: false })
    })
  }

  /// Open an existing log with the given options.
  ///
  /// **Note**: `BitcaskLog` constructed with this method is read only.
  pub fn open(cmp: C, opts: OpenOptions) -> io::Result<Self> {
    use std::fmt::Write;

    BUF.with(|buf| {
      let mut buf = buf.borrow_mut();
      buf.clear();
      write!(buf, "{:05}.{}", opts.fid, EXTENSION).unwrap();
      SkipMap::<Meta, C>::mmap_with_comparator(buf.as_str(), opts.lock, cmp)
        .map(|map| Self { map, fid: opts.fid, sync_on_write: false, ro: true })
    })
  }

  /// Inserts the given key and value to the log.
  #[inline]
  pub fn insert<'a, 'b: 'a>(&'a self, meta: Meta, key: &'b [u8], value: &'b [u8]) -> Result<Option<EntryRef<'a, C>>, Error> {
    match self.map.insert(meta, key, value) {
      Ok(ent) => {
        if self.sync_on_write {
          self.flush()?;
        }
        Ok(ent.map(EntryRef::new))
      },
      Err(e) => Err(Error::Log(e)),
    }
  }

  /// Inserts a new key if it does not yet exist. Returns `Ok(())` if the key was successfully inserted.
  ///
  /// This method is useful when you want to insert a key and you know the value size but you do not have the value
  /// at this moment.
  ///
  /// A placeholder value will be inserted first, then you will get an [`OccupiedValue`],
  /// and you must fully fill the value with bytes later in the closure.
  #[inline]
  pub fn insert_with<'a, 'b: 'a, E>(&'a self, meta: Meta, key: &'b [u8], value_size: u32, f: impl FnOnce(OccupiedValue<'a>) -> Result<(), E>) -> Result<Option<EntryRef<'a, C>>, Either<E, Error>> {
    match self.map.insert_with(meta, key, value_size, f) {
      Ok(ent) => {
        if self.sync_on_write {
          self.flush().map_err(|e| Either::Right(e.into()))?;
        }
        Ok(ent.map(EntryRef::new))
      },
      Err(e) => Err(e.map_right(Error::Log)),
    }
  }

  /// Inserts a batch of key-value pairs to the log.
  /// 
  /// ## Warning
  /// This method does not guarantee atomicity, which means that if the method fails in the middle of writing the batch,
  /// some of the key-value pairs may be written to the log.
  #[inline]
  pub fn insert_many(&self, batch: &[Entry]) -> Result<(), Error> {
    for (idx, ent) in batch.iter().enumerate() {
      self.map.insert(ent.meta, &ent.key, &ent.value).map_err(|e| Error::WriteBatch {
        idx,
        source: e,
      })?;
    }

    if self.sync_on_write {
      self.flush()?;
    }

    Ok(())
  }

  /// Gets the value associated with the given key.
  #[inline]
  pub fn get<'a, 'b: 'a>(&'a self, version: u64, key: &'b [u8]) -> Option<MapEntryRef<'a, Meta, C>> {
    self.map.get(version, key).and_then(|ent| {
      if ent.trailer().is_removed() {
        None
      } else {
        Some(ent)
      }
    })
  }

  /// Returns `true` if the log contains the given key.
  #[inline]
  pub fn contains_key(&self, version: u64, key: &[u8]) -> bool {
    self.get(version, key).is_some()
  }

  /// Returns the first (minimum) key in the log.
  #[inline]
  pub fn first(&self, version: u64) -> Option<EntryRef<C>> {
    self.map.first(version).map(EntryRef::new)
  }

  /// Returns the last (maximum) key in the log.
  #[inline]
  pub fn last(&self, version: u64) -> Option<EntryRef<C>> {
    self.map.last(version).map(EntryRef::new)
  }

  /// Returns an iterator over the entries less or equal to the given version in the log.
  #[inline]
  pub fn iter(&self, version: u64) -> BitcaskLogIterator<C> {
    BitcaskLogIterator { iter: self.map.iter(version), all_versions: false }
  }

  /// Returns an iterator over all versions of the entries less or equal to the given version in the log.
  #[inline]
  pub fn iter_all_versions(&self, version: u64) -> BitcaskLogIterator<C> {
    BitcaskLogIterator { iter: self.map.iter_all_versions(version), all_versions: true }
  }

  /// Returns a iterator that within the range, this iterator will yield the latest version of all entries in the range less or equal to the given version.
  #[inline]
  pub fn range<'a, Q, R>(&'a self, version: u64, range: R) -> BitcaskLogIterator<'a, C, Q, R>
  where
    &'a [u8]: PartialOrd<Q>,
    Q: ?Sized + PartialOrd<&'a [u8]>,
    R: RangeBounds<Q> + 'a,
  {
    BitcaskLogIterator { iter: self.map.range(version, range), all_versions: false }
  }

  /// Returns a iterator that within the range, this iterator will yield all versions of all entries in the range less or equal to the given version.
  #[inline]
  pub fn range_all_versions<'a, Q, R>(&'a self, version: u64, range: R) -> BitcaskLogIterator<'a, C, Q, R>
  where
    &'a [u8]: PartialOrd<Q>,
    Q: ?Sized + PartialOrd<&'a [u8]>,
    R: RangeBounds<Q> + 'a,
  {
    BitcaskLogIterator { iter: self.map.range_all_versions(version, range), all_versions: true }
  }
}

/// A reference to an entry in the log.
#[derive(Debug, Copy, Clone)]
pub struct EntryRef<'a, C> {
  ent: MapEntryRef<'a, Meta, C>,
}

impl<'a, C> EntryRef<'a, C> {
  /// Returns the key of the entry.
  #[inline]
  pub const fn key(&self) -> &[u8] {
    self.ent.key()
  }

  /// Returns the value of the entry.
  #[inline]
  pub const fn value(&self) -> &[u8] {
    self.ent.value()
  }

  /// Returns if the value of the entry is a value pointer.
  #[inline]
  pub const fn is_pointer(&self) -> bool {
    self.ent.trailer().is_pointer()
  }

  #[inline]
  const fn new(ent: MapEntryRef<'a, Meta, C>) -> Self {
    Self { ent }
  }
}

/// An entry in the log.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Entry {
  key: Bytes,
  value: Bytes,
  meta: Meta,
}

impl Entry {
  /// Create a new entry with the given key, value, and metadata.
  #[inline]
  pub const fn new(key: Bytes, value: Bytes, meta: Meta) -> Self {
    Self { key, value, meta }
  }

  /// Returns the key of the entry.
  #[inline]
  pub const fn key(&self) -> &Bytes {
    &self.key
  }

  /// Returns the value of the entry.
  #[inline]
  pub const fn value(&self) -> &Bytes {
    &self.value
  }

  /// Returns the metadata of the entry.
  #[inline]
  pub const fn meta(&self) -> Meta {
    self.meta
  }
}

#[derive(Clone, Copy)]
pub struct BitcaskLogIterator<'a, C, Q: ?Sized = &'static [u8], R = core::ops::RangeFull> {
  iter: skl::map::MapIterator<'a, Meta, C, Q, R>,
  all_versions: bool,
}

impl<'a, C: Comparator, Q, R> Iterator for BitcaskLogIterator<'a, C, Q, R>
where
  &'a [u8]: PartialOrd<Q>,
  Q: ?Sized + PartialOrd<&'a [u8]>,
  R: RangeBounds<Q>,
{
  type Item = EntryRef<'a, C>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.all_versions {
      return self.iter.next().map(EntryRef::new);
    }

    loop {
      match self.iter.next() {
        Some(ent) if !ent.trailer().is_removed() => return Some(EntryRef::new(ent)),
        None => return None,
        _ => {},
      }
    }
  }
}

impl<'a, C: Comparator, Q, R> DoubleEndedIterator for BitcaskLogIterator<'a, C, Q, R>
where
  &'a [u8]: PartialOrd<Q>,
  Q: ?Sized + PartialOrd<&'a [u8]>,
  R: RangeBounds<Q>,
{
  fn next_back(&mut self) -> Option<EntryRef<'a, C>> {
    if self.all_versions {
      return self.iter.next_back().map(EntryRef::new);
    }

    loop {
      match self.iter.next_back() {
        Some(ent) if !ent.trailer().is_removed() => return Some(EntryRef::new(ent)),
        None => return None,
        _ => {},
      }
    }
  }
}

impl<'a, C, Q, R> BitcaskLogIterator<'a, C, Q, R> {
  /// Returns the entry at the current position of the iterator.
  #[inline]
  pub fn entry(&self) -> Option<EntryRef<'a, C>> {
    self.iter.entry().map(|e| EntryRef::new(*e))
  }

  /// Returns the bounds of the iterator.
  #[inline]
  pub fn bounds(&self) -> &R {
    self.iter.bounds()
  }
}

impl<'a, C: Comparator, Q, R> BitcaskLogIterator<'a, C, Q, R>
where
  &'a [u8]: PartialOrd<Q>,
  Q: ?Sized + PartialOrd<&'a [u8]>,
  R: RangeBounds<Q>,
{
  /// Moves the iterator to the highest element whose key is below the given bound.
  /// If no such element is found then `None` is returned.
  pub fn seek_upper_bound(&mut self, upper: Bound<&[u8]>) -> Option<EntryRef<'a, C>> {
    if self.all_versions {
      return self.iter.seek_upper_bound(upper).map(EntryRef::new);
    }

    match self.iter.seek_upper_bound(upper) {
      Some(ent) if !ent.trailer().is_removed() => {
        return Some(EntryRef::new(ent));
      },
      None => None,
      _ => {
        loop {
          match self.iter.next_back() {
            Some(ent) if !ent.trailer().is_removed() => return Some(EntryRef::new(ent)),
            None => return None,
            _ => {},
          }
        }
      },
    }
  }

  /// Moves the iterator to the highest element whose key is below the given bound.
  /// If no such element is found then `None` is returned.
  pub fn seek_lower_bound(&mut self, lower: Bound<&[u8]>) -> Option<EntryRef<'a, C>> {
    if self.all_versions {
      return self.iter.seek_lower_bound(lower).map(EntryRef::new);
    }

    match self.iter.seek_lower_bound(lower) {
      Some(ent) if !ent.trailer().is_removed() => {
        return Some(EntryRef::new(ent));
      },
      None => None,
      _ => {
        loop {
          match self.iter.next() {
            Some(ent) if !ent.trailer().is_removed() => return Some(EntryRef::new(ent)),
            None => return None,
            _ => {},
          }
        }
      },
    }
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_meta() {
    let meta = Meta::new(0);
    assert_eq!(meta.version(), 0);
    assert!(!meta.is_removed());

    let meta = Meta::removed(1);
    assert_eq!(meta.version(), 1);
    assert!(meta.is_removed());

    let meta = Meta::new(100);
    assert_eq!(meta.version(), 100);
    assert!(!meta.is_removed());

    let meta = Meta::removed(101);
    assert_eq!(meta.version(), 101);
    assert!(meta.is_removed());

    assert_eq!(
      format!("{:?}", meta),
      "Meta { version: 101, removed: true, pointer: false }"
    );

    let meta = Meta::pointer(102);
    assert_eq!(meta.version(), 102);
    assert!(!meta.is_removed());
    assert!(meta.is_pointer());

    assert_eq!(
      format!("{:?}", meta),
      "Meta { version: 102, removed: false, pointer: true }"
    );
  }
}
