use super::*;
use core::cell::RefCell;
use std::io;

use skl::{SkipMap, Trailer, map::EntryRef as MapEntryRef};

pub use skl::{Ascend, Comparator, Descend};

const EXTENSION: &str = "skl";

/// The metadata for the skip log.
///
/// The metadata is a 64-bit value with the following layout:
///
/// ```text
/// +----------------------+--------------------------------+---------------------------+
/// | 62 bits for version  |  1 bit for value pointer mark  |  1 bit for deletion mark  |
/// +----------------------+--------------------------------+---------------------------+
/// ```
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Meta {
  /// 62 bits for version, 1 bit for value pointer mark, and 1 bit for deletion flag.
  meta: u64,
}

impl core::fmt::Debug for Meta {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Meta")
      .field("version", &self.version())
      .field("removed", &self.is_removed())
      .field("pointer", &self.is_pointer())
      .finish()
  }
}

impl Trailer for Meta {
  #[inline]
  fn version(&self) -> u64 {
    self.meta & Self::VERSION_MASK
  }
}

impl Meta {
  const VERSION_MASK: u64 = 0x3FFFFFFFFFFFFFFF; // 62 bits for version
  const VALUE_POINTER_FLAG: u64 = 1 << 62; // 63rd bit for value pointer mark
  const REMOVED_FLAG: u64 = 1 << 63; // 64th bit for removed mark

  /// Create a new metadata with the given version.
  #[inline]
  pub const fn new(version: u64) -> Self {
    assert!(version < Self::VERSION_MASK, "version is too large");

    Self { meta: version }
  }

  /// Create a new metadata with the given version and removed flag.
  #[inline]
  pub const fn removed(mut version: u64) -> Self {
    version |= Self::REMOVED_FLAG;
    Self { meta: version }
  }

  /// Create a new metadata with the given version and value pointer flag.
  #[inline]
  pub const fn pointer(mut version: u64) -> Self {
    version |= Self::VALUE_POINTER_FLAG;
    Self { meta: version }
  }

  /// Returns `true` if the entry is removed.
  #[inline]
  pub const fn is_removed(&self) -> bool {
    self.meta & Self::REMOVED_FLAG != 0
  }

  /// Returns `true` if the value of entry is a value pointer.
  #[inline]
  pub const fn is_pointer(&self) -> bool {
    self.meta & Self::VALUE_POINTER_FLAG != 0
  }
}

/// The options for creating a skip log.
#[viewit::viewit(getters(style = "move"), setters(prefix = "with"))]
pub struct CreateOptions {
  /// The file ID of the skip log.
  #[viewit(
    getter(const, attrs(doc = "Returns the file ID of the skip log.")),
    setter(attrs(doc = "Sets the file ID of the skip log."))
  )]
  fid: u32,

  /// The maximum size of the skip log.
  ///
  /// The skip log is backed by a mmaped file with the given size.
  /// So this size determines the mmaped file size.
  #[viewit(
    getter(const, attrs(doc = "Returns the size of the skip log.")),
    setter(attrs(doc = "Sets the size of the skip log."))
  )]
  size: usize,

  /// Whether to lock the skip log.
  ///
  /// If `true`, the skip log will be locked exlusively when it is created.
  #[viewit(
    getter(const, attrs(doc = "Returns if we should lock the skip log.")),
    setter(attrs(doc = "Sets whether to lock the skip log."))
  )]
  lock: bool,

  /// Whether to sync on write.
  /// 
  /// If `true`, the skip log will sync the data to disk on write.
  #[viewit(
    getter(const, attrs(doc = "Returns if we should sync on write.")),
    setter(attrs(doc = "Sets whether to sync on write."))
  )]
  sync_on_write: bool,
}

/// The options for opening a skip log.
#[viewit::viewit(getters(style = "move"), setters(prefix = "with"))]
pub struct OpenOptions {
  /// The file ID of the skip log.
  #[viewit(
    getter(const, attrs(doc = "Returns the file ID of the skip log.")),
    setter(attrs(doc = "Sets the file ID of the skip log."))
  )]
  fid: u32,

  /// Whether to lock the skip log.
  ///
  /// If `true`, the skip log will be locked exlusively when it is created.
  #[viewit(
    getter(const, attrs(doc = "Returns if we should lock the skip log.")),
    setter(attrs(doc = "Sets whether to lock the skip log."))
  )]
  lock: bool,
}

std::thread_local! {
  static BUF: RefCell<std::string::String> = RefCell::new(std::string::String::with_capacity(9));
}

/// Errors that can occur when working with a skip log.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// An I/O error occurred.
  #[error(transparent)]
  IO(#[from] io::Error),
  /// A skip log error occurred.
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

/// A write-ahead log based on on-disk [`SkipMap`].
pub struct SkipLog<C = Ascend> {
  map: SkipMap<Meta, C>,
  fid: u32,
  sync_on_write: bool,
  ro: bool,
}

impl<C> SkipLog<C> {
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

  /// Returns the file ID of the skip log.
  #[inline]
  pub const fn fid(&self) -> u32 {
    self.fid
  }

  /// Returns `true` if the skip log is read only.
  #[inline]
  pub const fn read_only(&self) -> bool {
    self.ro
  }

  /// Returns the current size of the skip log.
  #[inline]
  pub fn size(&self) -> usize {
    self.map.size()
  }

  /// Returns the capacity of the skip log.
  #[inline]
  pub fn capacity(&self) -> usize {
    self.map.capacity()
  }
}

impl<C: Comparator> SkipLog<C> {
  /// Create a new skip log with the given options.
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

  /// Open an existing skip log with the given options.
  ///
  /// **Note**: `SkipLog` constructed with this method is read only.
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

  /// Inserts the given key and value to the skip log.
  #[inline]
  pub fn insert(&self, meta: Meta, key: &[u8], value: &[u8]) -> Result<(), Error> {
    match self.map.insert(meta, key, value) {
      Ok(_) => {
        if self.sync_on_write {
          self.flush()?;
        }
        Ok(())
      },
      Err(e) => Err(Error::Log(e)),
    }
  }

  /// Inserts a batch of key-value pairs to the skip log.
  /// 
  /// ## Warning
  /// This method does not guarantee atomicity, which means that if the method fails in the middle of writing the batch,
  /// some of the key-value pairs may be written to the skip log.
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
  pub fn get<'a, 'b: 'a>(&'a self, version: u64, key: &'b [u8]) -> Option<MapEntryRef<'a, Meta>> {
    self.map.get(version, key).and_then(|ent| {
      if ent.trailer().is_removed() {
        None
      } else {
        Some(ent)
      }
    })
  }
}

/// A reference to an entry in the skip log.
#[derive(Debug, Copy, Clone)]
pub struct EntryRef<'a> {
  ent: MapEntryRef<'a, Meta>,
}

impl<'a> EntryRef<'a> {
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
}

/// An entry in the skip log.
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

// pub struct SkipLogIterator<'a, C> {
//   iter: skl::map::MapIterator<'a, Meta, C>,
// }

// impl<'a, C: Comparator> Iterator for SkipLogIterator<'a, C> {
//   type Item = EntryRef<'a>;

//   fn next(&mut self) -> Option<Self::Item> {
//     self.iter.next().map(|ent| EntryRef { ent })
//   }
// }


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
