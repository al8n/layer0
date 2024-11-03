use super::{hasher::SimMurmur, BloomHasher};

use core::f64::consts::LN_2;
use std::vec::Vec;

const CACHE_LINE_SIZE: usize = 64;
const CACHE_LINE_BITS: usize = CACHE_LINE_SIZE * 8;

#[inline]
const fn calculate_probes(bits_per_key: usize) -> u32 {
  // We intentionally round down to reduce probing cost a little bit
  let mut n = (bits_per_key as f64 * 0.69) as u32; // 0.69 ~= ln(2)
  if n < 1 {
    n = 1
  }

  if n > 30 {
    n = 30
  }

  n
}

/// Returns the bits per key required by bloomfilter based on
/// the false positive rate.
#[inline]
pub fn bits_per_key(num_entries: usize, fp: f64) -> usize {
  use libm::{ceil, log, pow};
  let size = -1.0 * num_entries as f64 * log(fp) / pow(LN_2, 2.0);
  ceil(LN_2 * size / num_entries as f64) as usize
}

/// A bloom filter builder.
#[derive(Debug, Clone)]
pub struct Filter<const N: usize = 128, S = SimMurmur> {
  bits_per_key: usize,

  num_hashes: usize,

  last_hash: u32,

  // We store the hashes in blocks.
  blocks: Vec<Vec<u32>>,

  hasher: S,
}

impl<const N: usize> Filter<N> {
  /// Creates a new filter builder.
  #[inline]
  pub fn new(num_entries: usize, fp: f64) -> Self {
    let bpk = bits_per_key(num_entries, fp);
    Self {
      bits_per_key: bpk,
      num_hashes: 0,
      last_hash: 0,
      blocks: Vec::new(),
      hasher: SimMurmur::new(),
    }
  }

  /// Creates a new filter builder.
  #[inline]
  pub const fn with_bits_per_key(bits_per_key: usize) -> Self {
    Self {
      bits_per_key,
      num_hashes: 0,
      last_hash: 0,
      blocks: Vec::new(),
      hasher: SimMurmur::new(),
    }
  }
}

impl<const N: usize, S> Filter<N, S> {
  /// Creates a new filter builder.
  #[inline]
  pub fn with_hasher(num_entries: usize, fp: f64, hasher: S) -> Self {
    let bpk = bits_per_key(num_entries, fp);
    Self {
      bits_per_key: bpk,
      num_hashes: 0,
      last_hash: 0,
      blocks: Vec::new(),
      hasher,
    }
  }

  /// Creates a new filter builder.
  #[inline]
  pub const fn with_bits_per_key_and_hasher(bits_per_key: usize, hasher: S) -> Self {
    Self {
      bits_per_key,
      num_hashes: 0,
      last_hash: 0,
      blocks: Vec::new(),
      hasher,
    }
  }
}

impl<const N: usize, S> Filter<N, S>
where
  S: BloomHasher,
{
  /// Adds a key to the filter.
  pub fn insert(&mut self, key: &[u8]) {
    let h = self.hasher.hash_one(key);
    if self.num_hashes != 0 && h == self.last_hash {
      return;
    }

    let ofs = self.num_hashes % N;
    if ofs == 0 {
      // Time for a new block
      self.blocks.push(std::vec![0; N]);
    }

    self
      .blocks
      .last_mut()
      .expect("blocks cannot be empty")
      .insert(ofs, h);
    self.last_hash = h;
    self.num_hashes += 1;
  }

  /// Returns the length of the final filter.
  #[inline]
  pub const fn filter_length(&self) -> usize {
    let n_lines = self.n_lines();
    // +5: 4 bytes for n_lines and 1 byte for n_probes
    n_lines * CACHE_LINE_SIZE + 5
  }

  const fn n_lines(&self) -> usize {
    let mut n_lines = 0;
    if self.num_hashes != 0 {
      n_lines = (self.num_hashes * self.bits_per_key).div_ceil(CACHE_LINE_BITS);
      // Make n_lines an odd number to make sure more bits are involved when
      // determining which block.
      if n_lines % 2 == 0 {
        n_lines += 1;
      }
    }

    // +5: 4 bytes for n_lines and 1 byte for n_probes
    n_lines
  }

  /// Finalize to the given buffer.
  ///
  /// ## Returns
  ///
  /// - Returns `Ok(usize)` the number of bytes written to the buffer.
  /// - Returns `Err(usize)` when the buf does not large enough to hold the filter, the number of bytes required to write the filter.
  pub fn finalize_to(self, buf: &mut [u8]) -> Result<usize, usize> {
    let n_lines = self.n_lines();
    let n_bytes = n_lines * CACHE_LINE_SIZE;
    let written = n_bytes + 5;
    if buf.len() < written {
      return Err(written);
    }

    self.finalize_in(n_lines, n_bytes, buf);
    Ok(written)
  }

  /// Finalizes the filter.
  pub fn finalize(self) -> std::vec::Vec<u8> {
    let n_lines = self.n_lines();
    let n_bytes = n_lines * CACHE_LINE_SIZE;
    // +5: 4 bytes for n_lines and 1 byte for n_probes
    let mut filter = std::vec![0; n_bytes + 5];
    self.finalize_in(n_lines, n_bytes, &mut filter);
    filter
  }

  fn finalize_in(mut self, n_lines: usize, n_bytes: usize, filter: &mut [u8]) {
    if n_lines != 0 {
      let n_probes = calculate_probes(self.bits_per_key);
      let num_blocks = self.blocks.len();
      for (bidx, b) in self.blocks.iter_mut().enumerate() {
        let mut length = N;
        if bidx == num_blocks - 1 && self.num_hashes % N != 0 {
          length = self.num_hashes % N;
        }

        for h in &mut b[..length] {
          let delta = h.rotate_left(15); // rotate right 17 bits
          let b = (*h % n_lines as u32) * CACHE_LINE_BITS as u32;

          for _ in 0..n_probes {
            let bit_pos = b + (*h % CACHE_LINE_BITS as u32);
            filter[(bit_pos / 8) as usize] |= 1 << (bit_pos % 8);
            *h = h.wrapping_add(delta);
          }
        }
      }

      filter[n_bytes] = n_probes as u8;
      filter[n_bytes + 1..n_bytes + 5].copy_from_slice((n_lines as u32).to_le_bytes().as_slice());
    }
  }
}

#[cfg(test)]
mod tests {
  #[cfg(feature = "xxhash3")]
  use crate::hasher::Xxh3;
  #[cfg(feature = "xxhash32")]
  use crate::hasher::Xxh32;

  use super::*;
  use crate::FrozenFilter;

  fn new_filter<'a, S: BloomHasher + Default>(
    bits_per_key: usize,
    keys: impl Iterator<Item = &'a [u8]>,
  ) -> std::vec::Vec<u8> {
    let mut builder =
      Filter::<512, S>::with_bits_per_key_and_hasher(bits_per_key, Default::default());
    for key in keys {
      builder.insert(key);
    }

    builder.finalize()
  }

  fn filter_to_string(src: &[u8]) -> String {
    let mut buf = String::new();

    for (i, x) in src.iter().enumerate() {
      if i > 0 {
        if i % 8 == 0 {
          buf.push('\n');
        } else {
          buf.push_str("  ");
        }
      }

      for j in 0..8 {
        if *x & (1 << (7 - j)) != 0 {
          buf.push('1');
        } else {
          buf.push('.');
        }
      }
    }

    buf.push('\n');
    buf
  }

  fn small_bloomfilter<S: BloomHasher + Default>(f: &[u8]) {
    let m = &[
      ("hello", true),
      ("world", true),
      ("x", false),
      ("foo", false),
    ];

    let f = FrozenFilter::with_hasher(f, S::default());
    for (key, want) in m {
      let got = f.may_contain(key.as_bytes());
      assert_eq!(got, *want);
    }
  }

  #[test]
  fn test_small_bloom_filter_simmurur() {
    let f = new_filter::<SimMurmur>(10, [b"hello", b"world"].iter().map(|e| e.as_slice()));

    let want = r###"
........  ........  ........  .......1  ........  ........  ........  ........
........  .1......  ........  .1......  ........  ........  ........  ........
...1....  ........  ........  ........  ........  ........  ........  ........
........  ........  ........  ........  ........  ........  ........  ...1....
........  ........  ........  ........  .....1..  ........  ........  ........
.......1  ........  ........  ........  ........  ........  .1......  ........
........  ........  ........  ........  ........  ...1....  ........  ........
.......1  ........  ........  ........  .1...1..  ........  ........  ........
.....11.  .......1  ........  ........  ........
"###;

    let want = want.trim_start();
    let got = filter_to_string(&f);
    for i in 0..want.len() {
      let goti = got.as_bytes()[i];
      let wanti = want.as_bytes()[i];
      assert_eq!(goti, wanti, "idx={i}");
    }

    small_bloomfilter::<SimMurmur>(&f);
  }

  #[test]
  #[cfg(feature = "xxhash32")]
  fn test_small_bloom_filter_xxhash32() {
    let f = new_filter::<Xxh32>(10, [b"hello", b"world"].iter().map(|e| e.as_slice()));
    small_bloomfilter::<Xxh32>(&f);
  }

  #[test]
  #[cfg(feature = "xxhash3")]
  fn test_small_bloom_filter_xxh3() {
    let f = new_filter::<Xxh3>(10, [b"hello", b"world"].iter().map(|e| e.as_slice()));
    small_bloomfilter::<Xxh3>(&f);
  }

  fn bloom_filter_in<S: BloomHasher + Default>() {
    let next_length = |x: usize| -> usize {
      if x < 10 {
        return x + 1;
      }

      if x < 100 {
        return x + 10;
      }

      if x < 1000 {
        return x + 100;
      }

      x + 1000
    };

    let le32 = |i: usize| -> [u8; 4] {
      let mut buf = [0; 4];
      buf[0] = (i as u32) as u8;
      buf[1] = ((i as u32) >> 8) as u8;
      buf[2] = ((i as u32) >> 16) as u8;
      buf[3] = ((i as u32) >> 24) as u8;
      buf
    };

    let (mut n_mediocre_filters, mut n_good_filters) = (0, 0);

    'l: loop {
      let mut length = 1;

      while length <= 10_000 {
        let keys = (0..length).map(&le32).collect::<std::vec::Vec<_>>();

        let f = new_filter::<S>(10, keys.iter().map(|b| b.as_slice()));
        // The size of the table bloom filter is measured in multiples of the
        // cache line size. The '+2' contribution captures the rounding up in the
        // length division plus preferring an odd number of cache lines. As such,
        // this formula isn't exact, but the exact formula is hard to read.
        let max_len = 5 + ((length * 10) / CACHE_LINE_BITS + 2) * CACHE_LINE_SIZE;
        if f.len() > max_len {
          #[cfg(feature = "std")]
          std::eprintln!(
            "length={}: f.len()={} > max len {}",
            length,
            f.len(),
            max_len
          );
          continue;
        }

        let f = FrozenFilter::with_hasher(f.as_slice(), S::default());
        // All added keys must match.
        for key in keys.iter() {
          if !f.may_contain(key) {
            #[cfg(feature = "std")]
            std::eprintln!("length={}: did not contain key {:?}", length, key);
            continue 'l;
          }
        }

        // Check false positive rate.
        let mut n_false_positive = 0f64;
        for i in 0..10_000 {
          if f.may_contain(le32((1e9f64 + i as f64) as usize).as_slice()) {
            n_false_positive += 1f64;
          }
        }

        if n_false_positive > 200f64 {
          #[cfg(feature = "std")]
          std::eprintln!(
            "length={}: n_false_positive={} > 0.02 * 10_000",
            length,
            n_false_positive
          );
          continue;
        }

        if n_false_positive > 125f64 {
          n_mediocre_filters += 1;
        } else {
          n_good_filters += 1;
        }

        length = next_length(length);
      }

      break;
    }

    if n_mediocre_filters > n_good_filters / 5 {
      #[cfg(feature = "std")]
      eprintln!(
        "{} mediocre filters buf only {} good filters",
        n_mediocre_filters, n_good_filters
      );
    }
  }

  #[test]
  fn bloom_filter_sim_murur() {
    bloom_filter_in::<SimMurmur>();
  }

  #[test]
  #[cfg(feature = "xxhash32")]
  fn bloom_filter_xxh32() {
    bloom_filter_in::<Xxh32>();
  }

  #[test]
  #[cfg(feature = "xxhash3")]
  fn bloom_filter_xxh3() {
    bloom_filter_in::<Xxh3>();
  }
}
