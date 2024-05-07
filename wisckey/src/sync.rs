use core::{fmt, marker::PhantomData};
use std::borrow::Cow;

use bytes::Bytes;
use either::Either;
use virtualfs::{File, SeekFrom};

use super::*;

// mod vlog;
// use vlog::Vlog;

// mod klog;
// use klog::Klog;

/// Error type for the value log.
pub enum Error<F: File, S: ValueSize> {
  /// File I/O error.
  File(F::Error),
  /// Value size encode/decode error.
  ValueSize(S::Error),
  /// Value size too large to be written to the value log.
  ValueTooLarge(u64),
}

impl<F: File, S: ValueSize> Error<F, S> {
  /// Create a new [`ValueTooLarge`] error.
  pub fn value_too_large(size: u64) -> Self {
    Self::ValueTooLarge(size)
  }
}

impl<F, S> fmt::Debug for Error<F, S>
where
  F: File,
  S: ValueSize,
  F::Error: fmt::Debug,
  S::Error: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::File(e) => write!(f, "{e:?}"),
      Self::ValueSize(e) => write!(f, "{e:?}"),
      Self::ValueTooLarge(size) => write!(
        f,
        "ValueTooLarge {{ actual: {size}, max: {} }}",
        S::MAX_SIZE
      ),
    }
  }
}

impl<F, S> fmt::Display for Error<F, S>
where
  F: File,
  S: ValueSize,
  F::Error: fmt::Display,
  S::Error: fmt::Display,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::File(e) => write!(f, "{e}"),
      Self::ValueSize(e) => write!(f, "{e}"),
      Self::ValueTooLarge(size) => write!(f, "value size too large: {size} > {}", S::MAX_SIZE),
    }
  }
}

#[cfg(feature = "std")]
impl<F, S> std::error::Error for Error<F, S>
where
  F: File,
  S: ValueSize,
  F::Error: fmt::Debug + fmt::Display,
  S::Error: fmt::Debug + fmt::Display,
{
}

/// The write ahead log abstraction.
pub trait Wal {
  /// The identifier type for this wal.
  type Id;
  /// The error type for this wal.
  type Error: crate::Error;
  /// The options used to open the wal.
  type Options;
  /// The value log type corresponding to this wal.
  type ValueLog: ValueLog;
  /// The reference for a key in the wal.
  type KeyRef<'a>
  where
    Self: 'a;

  /// The entry type for this wal.
  type Entry;
  /// The reference type for this wal.
  type EntryRef<'a>
  where
    Self: 'a;

  /// Open a wal with the given options.
  fn open(opts: Self::Options) -> Result<Self, Self::Error>
  where
    Self: Sized;

  /// Returns the length of the wal.
  fn len(&self) -> u64;

  /// Returns `true` if the wal is empty.
  fn is_empty(&self) -> bool;

  /// Returns `true` if the wal is full.
  fn is_full(&self) -> bool;

  /// Read a key from the wal.
  fn read(&self, key: Self::KeyRef<'_>) -> Result<Self::EntryRef<'_>, Self::Error>;

  /// Write an entry to the log.
  fn write(&mut self, ent: Self::Entry) -> Result<(), Self::Error>;

  /// Flush the wal.
  fn flush(&mut self) -> Result<(), Self::Error>;
}

/// Value log abstraction.
pub trait ValueLog {
  /// The identifier type for this value log.
  type Id;
  /// The value size for this value log.
  type Size: ValueSize;
  /// The error type for this value log.
  type Error: crate::Error;
  /// The options used to open the value log.
  type Options;
  /// The reference for a value in the value log.
  type ValueRef<'a>
  where
    Self: 'a;

  /// Open a value log with the given options.
  fn open(opts: Self::Options) -> Result<Self, Self::Error>
  where
    Self: Sized;

  /// Returns the length of the value log.
  fn len(&self) -> u64;

  /// Returns `true` if the value log is empty.
  fn is_empty(&self) -> bool;

  /// Returns the maximum size of the value log
  fn max_size(&self) -> u64;

  /// Returns the current position of the value log.
  fn position(&self) -> u64;

  /// Seek the value log to the specified position.
  fn seek(&mut self, pos: SeekFrom) -> Result<(), Self::Error>;

  /// Read a value from the value log.
  fn read(&self, offset: u64, size: Self::Size) -> Result<Self::ValueRef<'_>, Self::Error>;

  /// Write a value to the value log.
  ///
  /// Returns a [`ValuePointer`] that can be used to retrieve the value later.
  fn write(&mut self, value: &[u8]) -> Result<ValuePointer<Self::Id, Self::Size>, Self::Error>;

  /// Flush the value log.
  fn flush(&mut self) -> Result<(), Self::Error>;
}

/// An abstraction for the WiscKey (key-value seperate write ahead log).
///
/// The implementation of this trait should keep in mind that one write ahead log
/// may have multiple value logs.
pub trait WiscKey {
  /// The write ahead log type.
  type Wal: Wal<ValueLog = Self::ValueLog>;

  /// The value log type.
  type ValueLog: ValueLog;

  /// The error type for this WiscKey.
  type Error: crate::Error;

  /// The options used to open the WiscKey.
  type Options;

  /// Open a WiscKey with the given options.
  fn open(opts: Self::Options) -> Result<Self, Self::Error>
  where
    Self: Sized;

  /// Open a value log with the given options.
  fn open_value_log(
    opts: <Self::ValueLog as ValueLog>::Options,
  ) -> Result<Self::ValueLog, Self::Error>;

  /// Returns the number of value logs
  fn num_value_logs(&self) -> usize;

  /// Returns the threshold for the value log, which implies that if the size of a value
  /// is greater than this threshold, it will be written to the value log instead of the wal.
  fn value_threshold(&self) -> u64;

  /// Write a key-value pair to the log.
  fn write(&mut self, ent: <Self::Wal as Wal>::Entry) -> Result<(), Self::Error>;

  /// Write a batch of key-value pairs to the log.
  fn write_batch(
    &mut self,
    batch: impl Iterator<Item = <Self::Wal as Wal>::Entry>,
  ) -> Result<(), Self::Error>;

  /// Read a key from the log.
  fn read(&self, key: <Self::Wal as Wal>::KeyRef<'_>)
    -> Result<<Self::Wal as Wal>::EntryRef<'_>, Self::Error>;

  /// Read value
  fn read_pointer(
    &self,
    pointer: ValuePointer<<Self::ValueLog as ValueLog>::Id, <Self::ValueLog as ValueLog>::Size>,
  ) -> Result<<Self::ValueLog as ValueLog>::ValueRef<'_>, Self::Error>;
}
