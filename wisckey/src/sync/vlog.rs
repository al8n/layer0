use super::*;

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

  /// Returns the current position of the value log.
  #[inline]
  pub fn position(&self) -> u64 {
    self.vlog.position()
  }

  /// Seek the value log to the specified position.
  #[inline]
  pub fn seek(&mut self, pos: SeekFrom) -> Result<(), Error<F, S>> {
    self.vlog.seek(pos).map_err(Error::File)
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
      .map(|_| ValuePointer {
        fid,
        offset: cur,
        size,
      })
      .map_err(Error::File)
  }

  /// Flush the value log to the disk.
  #[inline]
  pub fn flush(&mut self) -> Result<(), Error<F, S>> {
    self.vlog.flush().map_err(Error::File)
  }
}
