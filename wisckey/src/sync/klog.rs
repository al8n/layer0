use either::Either;

use super::*;

/// A simple value log.
pub struct Klog<F, S> {
  _m: PhantomData<S>,
}

impl<F, S> Klog<F, S>
where
  F: File,
  S: ValueSize,
{
  /// Open a value log with the given options.
  #[inline]
  pub fn open(opts: F::Options) -> Result<Self, Error<F, S>> {
    todo!()
  }

  /// Returns the length of the value log.
  #[inline]
  pub fn len(&self) -> u64 {
    self.klog.len()
  }

  /// Returns `true` if the value log is empty.
  #[inline]
  pub fn is_empty(&self) -> bool {
    self.klog.is_empty()
  }

  /// Seek the value log to the specified position.
  #[inline]
  pub fn seek(&mut self, pos: SeekFrom) -> Result<(), Error<F, S>> {
    self.klog.seek(pos).map_err(Error::File)
  }

  /// Read a value from the value log.
  #[inline]
  pub fn read(
    &mut self,
    key: &[u8],
  ) -> Result<Either<Cow<'_, [u8]>, ValuePointer<F::Id, S>>, Error<F, S>> {
    todo!()
  }
}

impl<F, S> Klog<F, S>
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
    let cur = self.klog.position();
    let fid = self.klog.id().clone();

    self
      .klog
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
    self.klog.flush().map_err(Error::File)
  }
}
