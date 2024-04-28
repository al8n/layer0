/// The file abstraction
pub trait AsyncFile: Sized + 'static {
  /// The metadata type for this file.
  type Meta;

  /// The error type for this file.
  type Error;

  /// The options type for this file.
  type Options;

  /// Open a file
  fn open(opts: Self::Options) -> Result<Self, Self::Error>;

  /// Returns the metadata for the file.
  fn metadata(&self) -> Result<Self::Meta, Self::Error>;

  /// Lock the file.
  fn lock(&self) -> Result<(), Self::Error>;

  /// Unlock the file.
  fn unlock(&self) -> Result<(), Self::Error>;

  /// Read data from the file.
  fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;

  /// Read exact data from the file.
  fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error>;

  /// Write data to the file.
  fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error>;

  /// Write all the data to the file.
  fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error>;

  /// Length of the file.
  fn len(&self) -> u64;

  /// Returns `true` if the file is empty.
  fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

/// Abstraction of file system
pub trait AsyncFileSystem: Sized + 'static {
  /// The error type for this file system.
  type Error;

  /// The directory type for this file system.
  type Directory;

  /// The file type for this file system.
  type File: AsyncFile;

  /// The options type for this file system.
  type Options;

  /// Create a new file system.
  fn create(opts: Self::Options) -> Result<Self, Self::Error>;
}
