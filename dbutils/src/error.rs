#[derive(Debug, Clone, PartialEq, Eq)]
struct Information {
  required: u64,
  remaining: u64,
}

/// Returned when the encoded buffer is too small to hold the bytes format of the types.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct InsufficientBuffer {
  info: Option<Information>,
}

impl InsufficientBuffer {
  /// Creates a new instance of the error.
  #[inline]
  pub const fn new() -> Self {
    Self { info: None }
  }

  /// Creates a new instance of the error with size information.
  #[inline]
  pub const fn with_information(required: u64, remaining: u64) -> Self {
    Self {
      info: Some(Information {
        required,
        remaining,
      }),
    }
  }

  /// Returns the required size.
  #[inline]
  pub fn required(&self) -> Option<u64> {
    self.info.as_ref().map(|info| info.required)
  }

  /// Returns the remaining size.
  #[inline]
  pub fn remaining(&self) -> Option<u64> {
    self.info.as_ref().map(|info| info.remaining)
  }
}

impl core::fmt::Display for InsufficientBuffer {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self.info {
      Some(ref info) => {
        write!(
          f,
          "incomplete buffer data: expected {} bytes for decoding, but only {} bytes were available",
          info.required, info.remaining
        )
      }
      None => {
        write!(
          f,
          "the buffer did not have enough space to encode the value"
        )
      }
    }
  }
}

impl core::error::Error for InsufficientBuffer {}

/// Returned when the buffer does not contains engouth bytes for decoding.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct IncompleteBuffer {
  info: Option<Information>,
}

impl IncompleteBuffer {
  /// Creates a new instance of the error.
  #[inline]
  pub const fn new() -> Self {
    Self { info: None }
  }

  /// Creates a new instance of the error with size information.
  #[inline]
  pub const fn with_information(required: u64, remaining: u64) -> Self {
    Self {
      info: Some(Information {
        required,
        remaining,
      }),
    }
  }

  /// Returns the required size.
  #[inline]
  pub fn required(&self) -> Option<u64> {
    self.info.as_ref().map(|info| info.required)
  }

  /// Returns the remaining size.
  #[inline]
  pub fn remaining(&self) -> Option<u64> {
    self.info.as_ref().map(|info| info.remaining)
  }
}

impl core::fmt::Display for IncompleteBuffer {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self.info {
      Some(ref info) => {
        write!(
          f,
          "incomplete buffer data: expected {} bytes for decoding, but only {} bytes were available",
          info.required, info.remaining
        )
      }
      None => {
        write!(
          f,
          "the buffer did not contain enough bytes to decode a value"
        )
      }
    }
  }
}

impl core::error::Error for IncompleteBuffer {}
