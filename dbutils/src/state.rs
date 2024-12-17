/// The state for the entry.
pub trait State {
  /// The data type of this state.
  type Data<'a, T>;

  /// Construct the data of the state from the given `T`.
  fn data<'a, T>(data: T) -> Self::Data<'a, T>;
}

/// A state for the entry that is active.
pub struct Active;

impl State for Active {
  type Data<'a, T> = T;

  #[inline]
  fn data<'a, T>(data: T) -> Self::Data<'a, T> {
    data
  }
}

/// A state for the entry that may be a tombstone.
pub struct MaybeTombstone;

impl State for MaybeTombstone {
  type Data<'a, T> = Option<T>;

  #[inline]
  fn data<'a, T>(data: T) -> Self::Data<'a, T> {
    Some(data)
  }
}
