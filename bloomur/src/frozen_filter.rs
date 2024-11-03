use super::{hasher::SimMurmur, BloomHasher};

/// A frozen filter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrozenFilter<A, S = SimMurmur> {
  src: A,
  hasher: S,
}

impl<A> From<A> for FrozenFilter<A> {
  #[inline]
  fn from(a: A) -> Self {
    Self {
      src: a,
      hasher: SimMurmur::new(),
    }
  }
}

impl<A> FrozenFilter<A> {
  /// Creates a new frozen filter with the default hasher.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use bloomur::{Filter, FrozenFilter};
  ///
  /// let mut filter = Filter::<512>::new(10_000, 0.01);
  ///
  /// filter.insert(b"hello");
  /// filter.insert(b"world");
  ///
  /// let b = filter.finalize();
  ///
  /// let frozen = FrozenFilter::<&[u8]>::from(b.as_ref());
  ///
  /// assert!(frozen.may_contain(b"hello"));
  /// assert!(frozen.may_contain(b"world"));
  /// assert!(!frozen.may_contain(b"foo"));
  ///
  /// let frozen = FrozenFilter::new(b);
  ///
  /// assert!(frozen.may_contain(b"hello"));
  /// assert!(frozen.may_contain(b"world"));
  /// assert!(!frozen.may_contain(b"foo"));
  /// ```
  #[inline]
  pub const fn new(a: A) -> Self {
    Self {
      src: a,
      hasher: SimMurmur::new(),
    }
  }
}

impl<A, S> FrozenFilter<A, S> {
  /// Creates a new frozen filter with the given hasher.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use bloomur::{Filter, FrozenFilter, hasher::SimMurmur};
  ///
  /// let mut filter = Filter::<512>::new(10_000, 0.01);
  ///
  /// filter.insert(b"hello");
  /// filter.insert(b"world");
  ///
  /// let b = filter.finalize();
  ///
  /// let frozen = FrozenFilter::<&[u8]>::with_hasher(b.as_ref(), SimMurmur::new());
  ///
  /// assert!(frozen.may_contain(b"hello"));
  /// assert!(frozen.may_contain(b"world"));
  /// assert!(!frozen.may_contain(b"foo"));
  /// ```
  #[inline]
  pub const fn with_hasher(a: A, hasher: S) -> Self {
    Self { src: a, hasher }
  }
}

impl<A: AsRef<[u8]>, S: BloomHasher> FrozenFilter<A, S> {
  /// Returns `true` if the filter may contain the key.
  #[inline]
  pub fn may_contain(&self, key: &[u8]) -> bool {
    let filter = self.src.as_ref();
    let len = filter.len();
    if len <= 5 {
      return false;
    }

    let n = len - 5;
    let n_probes = filter[n];
    let n_lines = u32::from_le_bytes([filter[n + 1], filter[n + 2], filter[n + 3], filter[n + 4]]);
    let cache_line_bits = 8 * ((n as u32) / n_lines);

    let mut h = self.hasher.hash_one(key);
    let delta = h.rotate_left(15);
    let b = (h % n_lines) * cache_line_bits;

    let mut j = 0;
    while j < n_probes {
      let bit_pos = b + (h % cache_line_bits);
      if filter[(bit_pos / 8) as usize] & (1 << (bit_pos % 8)) == 0 {
        return false;
      }
      h = h.wrapping_add(delta);
      j += 1;
    }

    true
  }
}
