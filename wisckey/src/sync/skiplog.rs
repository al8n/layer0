use super::*;
use core::cell::RefCell;
use std::io;

use skl::{SkipMap, Trailer};

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

/// A write-ahead log based on on-disk [`SkipMap`].
pub struct SkipLog<C = Ascend> {
  map: SkipMap<Meta, C>,
  fid: u32,
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
        .map(|map| Self { map, fid: opts.fid })
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
        .map(|map| Self { map, fid: opts.fid })
    })
  }

  /// Returns the file ID of the skip log.
  #[inline]
  pub const fn fid(&self) -> u32 {
    self.fid
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
