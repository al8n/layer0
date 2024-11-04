/// Synchronous watermark implementation.
#[cfg(feature = "sync")]
pub mod sync;

/// Asynchronous watermark implementation.
#[cfg(feature = "future")]
#[cfg_attr(docsrs, doc(cfg(feature = "future")))]
pub mod future;

/// Error type for watermark.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WaterMarkError {
  /// The watermark is uninitialized, please call init first before using any other functions
  Uninitialized,
  /// The watermark is canceled.
  Canceled,
  /// The channel is closed.
  ChannelClosed,
}

impl core::fmt::Display for WaterMarkError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Uninitialized => write!(
        f,
        "watermark: uninitialized, please call init first before using any other functions"
      ),
      Self::Canceled => write!(f, "watermark: canceled"),
      Self::ChannelClosed => write!(f, "watermark: channel closed"),
    }
  }
}

impl core::error::Error for WaterMarkError {}

#[test]
#[cfg(any(feature = "alloc", feature = "std"))]
fn test_error() {
  assert_eq!(
    std::format!("{}", WaterMarkError::Uninitialized),
    "watermark: uninitialized, please call init first before using any other functions"
  );

  assert_eq!(
    std::format!("{}", WaterMarkError::Canceled),
    "watermark: canceled"
  );

  assert_eq!(
    std::format!("{}", WaterMarkError::ChannelClosed),
    "watermark: channel closed"
  );
}
