use core::{fmt, marker::PhantomData};
use std::borrow::Cow;

use virtualfs::{File, SeekFrom};

use super::*;

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

/// A simple value log.
pub struct Vlog<F, S> {
  vlog: F,
  _m: PhantomData<S>,
}

impl<F, S> Vlog<F, S>
where
  F: File,
  S: ValueSize,
{
  /// Open a value log with the given options.
  #[inline]
  pub fn open(opts: F::Options) -> Result<Self, Error<F, S>> {
    F::open(opts)
      .map(|f| Self {
        vlog: f,
        _m: PhantomData,
      })
      .map_err(Error::File)
  }

  /// Returns the length of the value log.
  #[inline]
  pub fn len(&self) -> u64 {
    self.vlog.len()
  }

  /// Returns `true` if the value log is empty.
  #[inline]
  pub fn is_empty(&self) -> bool {
    self.vlog.is_empty()
  }

  /// Read a value from the value log.
  #[inline]
  pub fn read(&mut self, offset: u64, size: S) -> Result<Cow<'_, [u8]>, Error<F, S>> {
    self
      .vlog
      .seek(SeekFrom::Start(offset))
      .and_then(|_| self.vlog.read_at(size.into_usize()))
      .map_err(Error::File)
  }
}

impl<F, S> Vlog<F, S>
where
  F: File,
  F::Id: Clone,
  S: ValueSize,
{
  /// Write a value to the value log.
  ///
  /// Returns a [`ValuePointer`] that can be used to retrieve the value later.
  pub fn write(&mut self, value: &[u8]) -> Result<ValuePointer<F::Id, S>, Error<F, S>> {
    let value_size = value.len();
    if value_size > S::MAX_SIZE {
      return Err(Error::value_too_large(value_size as u64));
    }

    let size = S::from_usize(value_size);
    let cur = self.vlog.position();
    let fid = self.vlog.id().clone();

    self
      .vlog
      .write_all(value)
      .and_then(|_| {
        self.vlog.flush().map(|_| ValuePointer {
          fid,
          offset: cur,
          size,
        })
      })
      .map_err(Error::File)
  }
}
