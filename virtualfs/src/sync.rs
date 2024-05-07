use std::borrow::Cow;

use super::*;

/// The file abstraction
pub trait File: Sized + 'static {
  /// The unique identifier for this file.
  type Id;

  /// The metadata type for this file.
  type Meta;

  /// The error type for this file.
  type Error;

  /// The options type for this file.
  type Options;

  /// Returns the unique identifier for this file.
  fn id(&self) -> &Self::Id;

  /// Open a file
  fn open(opts: Self::Options) -> Result<Self, Self::Error>;

  /// Returns the metadata for the file.
  fn metadata(&self) -> Result<Self::Meta, Self::Error>;

  /// Lock the file.
  fn lock(&self) -> Result<(), Self::Error>;

  /// Unlock the file.
  fn unlock(&self) -> Result<(), Self::Error>;

  /// Read data from the file into the buffer.
  fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;

  /// Read exact data from the file.
  fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error>;

  /// Read data from the file at the current position.
  fn read_at(&mut self, size: usize) -> Result<Cow<'_, [u8]>, Self::Error>;

  /// Write data to the file.
  fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error>;

  /// Write all the data to the file.
  fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error>;

  /// Flush the file.
  fn flush(&mut self) -> Result<(), Self::Error>;

  /// Seek to a position in the file.
  fn seek(&mut self, pos: SeekFrom) -> Result<(), Self::Error>;

  /// Seek to the start of the file.
  fn rewind(&mut self) -> Result<(), Self::Error> {
    self.seek(SeekFrom::Start(0))
  }

  /// Seek to the end of the file.
  fn seek_to_end(&mut self) -> Result<(), Self::Error> {
    self.seek(SeekFrom::End(0))
  }

  /// Returns the current position of the file
  fn position(&self) -> u64;

  /// Length of the file.
  fn len(&self) -> u64;

  /// Returns `true` if the file is empty.
  fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

/// Abstraction of file system
pub trait FileSystem: Sized + 'static {
  /// The error type for this file system.
  type Error;

  /// The directory type for this file system.
  type Directory;

  /// The file type for this file system.
  type File: File;

  /// The options type for this file system.
  type Options;

  /// Create a new file system.
  fn create(opts: Self::Options) -> Result<Self, Self::Error>;
}
