use super::BloomHasher;

/// A hasher that similiar to the Murmur hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimMurmur {
  seed: u32,
}

impl Default for SimMurmur {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl BloomHasher for SimMurmur {
  fn hash_one(&self, src: &[u8]) -> u32 {
    const M: u32 = 0xc6a4a793;

    let mut src_len = src.len();
    let mut h = self.seed ^ ((src_len as u32).wrapping_mul(M));

    let mut cursor = 0;

    while src_len >= 4 {
      let k = (src[cursor] as u32)
        | ((src[cursor + 1] as u32) << 8)
        | ((src[cursor + 2] as u32) << 16)
        | ((src[cursor + 3] as u32) << 24);
      h = h.wrapping_add(k);
      h = h.wrapping_mul(M);
      h ^= h >> 16;
      src_len -= 4;
      cursor += 4;
    }

    // Handle the remaining bytes, with sign-extension as required.
    // This converts each byte to i8 to match RocksDB behavior.
    match src_len {
      3 => {
        h = h.wrapping_add((src[cursor + 2] as i8 as u32) << 16);
        h = h.wrapping_add((src[cursor + 1] as i8 as u32) << 8);
        h = h.wrapping_add(src[cursor] as i8 as u32);
        h = h.wrapping_mul(M);
        h ^= h >> 24;
      }
      2 => {
        h = h.wrapping_add((src[cursor + 1] as i8 as u32) << 8);
        h = h.wrapping_add(src[cursor] as i8 as u32);
        h = h.wrapping_mul(M);
        h ^= h >> 24;
      }
      1 => {
        h = h.wrapping_add(src[cursor] as i8 as u32);
        h = h.wrapping_mul(M);
        h ^= h >> 24;
      }
      _ => {}
    };

    h
  }
}

impl SimMurmur {
  /// Creates a new `SimMurmur` hasher.
  #[inline]
  pub const fn new() -> Self {
    Self { seed: 0xbc9f1d34 }
  }

  /// Creates a new `SimMurmur` with a seed.
  #[inline]
  pub const fn with_seed(seed: u32) -> Self {
    Self { seed }
  }
}

#[test]
fn hash() {
  // The magic expected numbers come from RocksDB's util/hash_test.cc:TestHash.
  const TEST_CASES: &[(&[u8], u32)] = &[
    (b"", 3164544308),
    (b"\x08", 422599524),
    (b"\x17", 3168152998),
    (b"\x9a", 3195034349),
    (b"\x1c", 2651681383),
    (b"\x4d\x76", 2447836956),
    (b"\x52\xd5", 3854228105),
    (b"\x91\xf7", 31066776),
    (b"\xd6\x27", 1806091603),
    (b"\x30\x46\x0b", 3808221797),
    (b"\x56\xdc\xd6", 2157698265),
    (b"\xd4\x52\x33", 1721992661),
    (b"\x6a\xb5\xf4", 2469105222),
    (b"\x67\x53\x81\x1c", 118283265),
    (b"\x69\xb8\xc0\x88", 3416318611),
    (b"\x1e\x84\xaf\x2d", 3315003572),
    (b"\x46\xdc\x54\xbe", 447346355),
    (b"\xd0\x7a\x6e\xea\x56", 4255445370),
    (b"\x86\x83\xd5\xa4\xd8", 2390603402),
    (b"\xb7\x46\xbb\x77\xce", 2048907743),
    (b"\x6c\xa8\xbc\xe5\x99", 2177978500),
    (b"\x5c\x5e\xe1\xa0\x73\x81", 1036846008),
    (b"\x08\x5d\x73\x1c\xe5\x2e", 229980482),
    (b"\x42\xfb\xf2\x52\xb4\x10", 3655585422),
    (b"\x73\xe1\xff\x56\x9c\xce", 3502708029),
    (b"\x5c\xbe\x97\x75\x54\x9a\x52", 815120748),
    (b"\x16\x82\x39\x49\x88\x2b\x36", 3056033698),
    (b"\x59\x77\xf0\xa7\x24\xf4\x78", 587205227),
    (b"\xd3\xa5\x7c\x0e\xc0\x02\x07", 2030937252),
    (b"\x31\x1b\x98\x75\x96\x22\xd3\x9a", 469635402),
    (b"\x38\xd6\xf7\x28\x20\xb4\x8a\xe9", 3530274698),
    (b"\xbb\x18\x5d\xf4\x12\x03\xf7\x99", 1974545809),
    (b"\x80\xd4\x3b\x3b\xae\x22\xa2\x78", 3563570120),
    (b"\x1a\xb5\xd0\xfe\xab\xc3\x61\xb2\x99", 2706087434),
    (b"\x8e\x4a\xc3\x18\x20\x2f\x06\xe6\x3c", 1534654151),
    (b"\xb6\xc0\xdd\x05\x3f\xc4\x86\x4c\xef", 2355554696),
    (b"\x9a\x5f\x78\x0d\xaf\x50\xe1\x1f\x55", 1400800912),
    (b"\x22\x6f\x39\x1f\xf8\xdd\x4f\x52\x17\x94", 3420325137),
    (b"\x32\x89\x2a\x75\x48\x3a\x4a\x02\x69\xdd", 3427803584),
    (b"\x06\x92\x5c\xf4\x88\x0e\x7e\x68\x38\x3e", 1152407945),
    (b"\xbd\x2c\x63\x38\xbf\xe9\x78\xb7\xbf\x15", 3382479516),
  ];

  let hash = SimMurmur::new();
  for (src, expected) in TEST_CASES {
    assert_eq!(hash.hash_one(src), *expected);
  }
}
