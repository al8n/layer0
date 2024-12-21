/// The state for the entry.
pub trait State {
  /// `true` if the data of this state is never invalid.
  const ALWAYS_VALID: bool;

  /// The data type of this state.
  type Data<'a, T>;

  /// Construct the data of the state from the given `T`.
  fn data<'a, T>(data: T) -> Self::Data<'a, T>;

  /// Returns `true` if the state is valid.
  fn validate_data<T>(data: &Self::Data<'_, T>) -> bool;
}

/// A state for the entry that is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Active;

impl State for Active {
  const ALWAYS_VALID: bool = true;

  type Data<'a, T> = T;

  #[inline]
  fn data<'a, T>(data: T) -> Self::Data<'a, T> {
    data
  }

  #[inline(always)]
  fn validate_data<T>(_: &Self::Data<'_, T>) -> bool {
    true
  }
}

/// A state for the entry that may be a tombstone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaybeTombstone;

impl State for MaybeTombstone {
  const ALWAYS_VALID: bool = false;

  type Data<'a, T> = Option<T>;

  #[inline]
  fn data<'a, T>(data: T) -> Self::Data<'a, T> {
    Some(data)
  }

  #[inline(always)]
  fn validate_data<T>(data: &Self::Data<'_, T>) -> bool {
    data.is_some()
  }
}
