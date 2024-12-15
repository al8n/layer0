use core::ops::RangeBounds;

use super::{Builder, Entry, Rewindable, Seekable};

pub trait SealedRange<Q, R, E>
where
  E: ?Sized,
  Q: ?Sized,
  R: RangeBounds<Q>,
{
  type Initializor;
  type KeyValidator;
  type ValueValidator;
  type Comparator;

  fn range(
    version: E::Version,
    range: R,
    builder: Builder<Self::Initializor, Self::Comparator, Self::KeyValidator, Self::ValueValidator>,
  ) -> Self
  where
    E: Entry,
    Self: Sized,
    Self::Initializor: Seekable<Q, Entry = E>;
}

pub trait SealedIter<E: ?Sized> {
  type Initializor;
  type KeyValidator;
  type ValueValidator;
  type Comparator;

  fn new(
    version: E::Version,
    builder: Builder<Self::Initializor, Self::Comparator, Self::KeyValidator, Self::ValueValidator>,
  ) -> Self
  where
    E: Entry,
    Self: Sized,
    Self::Initializor: Rewindable<Entry = E>;
}
