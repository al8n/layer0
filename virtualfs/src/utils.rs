//! The code in this mod is copied and modified base on [prost]
//!
//! [prost]: https://github.com/tokio-rs/prost/blob/master/prost/src/encoding.rs.

/// Error type for encoding and decoding varints.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum VarintError {
  /// Invalid varint encoding.
  Invalid,

  /// Encode buffer too small.
  BufferTooSmall,
}

impl core::fmt::Display for VarintError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Invalid => write!(f, "invalid varint encoding"),
      Self::BufferTooSmall => write!(f, "encode buffer too small"),
    }
  }
}

#[cfg(feature = "std")]
impl std::error::Error for VarintError {}

/// Encodes an integer value into LEB128 variable length format, and writes it to the buffer.
/// The buffer must have enough remaining space (maximum 10 bytes).
/// Encodes an integer value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub fn encode_varint(mut x: u64, buf: &mut [u8]) -> Result<usize, VarintError> {
  let mut i = 0;

  while x >= 0x80 {
    if i >= buf.len() {
      return Err(VarintError::BufferTooSmall);
    }

    buf[i] = (x as u8) | 0x80;
    x >>= 7;
    i += 1;
  }
  buf[i] = x as u8;
  Ok(i + 1)
}

/// Decodes a LEB128-encoded variable length integer from the slice, returning the value and the
/// number of bytes read.
///
/// Based loosely on [`ReadVarint64FromArray`][1] with a varint overflow check from
/// [`ConsumeVarint`][2].
///
/// ## Safety
///
/// The caller must ensure that `bytes` is non-empty and either `bytes.len() >= 10` or the last
/// element in bytes is < `0x80`.
///
/// [1]: https://github.com/google/protobuf/blob/3.3.x/src/google/protobuf/io/coded_stream.cc#L365-L406
/// [2]: https://github.com/protocolbuffers/protobuf-go/blob/v1.27.1/encoding/protowire/wire.go#L358
#[inline]
pub fn decode_varint(bytes: &[u8]) -> Result<(usize, u64), VarintError> {
  // Fully unrolled varint decoding loop. Splitting into 32-bit pieces gives better performance.

  if bytes.is_empty() {
    return Err(VarintError::Invalid);
  }

  if bytes.len() > 10 || bytes[bytes.len() - 1] < 0x80 {
    return Err(VarintError::Invalid);
  }

  let mut b: u8 = unsafe { *bytes.get_unchecked(0) };
  let mut part0: u32 = u32::from(b);
  if b < 0x80 {
    return Ok((1, u64::from(part0)));
  };
  part0 -= 0x80;
  b = unsafe { *bytes.get_unchecked(1) };
  part0 += u32::from(b) << 7;
  if b < 0x80 {
    return Ok((2, u64::from(part0)));
  };
  part0 -= 0x80 << 7;
  b = unsafe { *bytes.get_unchecked(2) };
  part0 += u32::from(b) << 14;
  if b < 0x80 {
    return Ok((3, u64::from(part0)));
  };
  part0 -= 0x80 << 14;
  b = unsafe { *bytes.get_unchecked(3) };
  part0 += u32::from(b) << 21;
  if b < 0x80 {
    return Ok((4, u64::from(part0)));
  };
  part0 -= 0x80 << 21;
  let value = u64::from(part0);

  b = unsafe { *bytes.get_unchecked(4) };
  let mut part1: u32 = u32::from(b);
  if b < 0x80 {
    return Ok((5, value + (u64::from(part1) << 28)));
  };
  part1 -= 0x80;
  b = unsafe { *bytes.get_unchecked(5) };
  part1 += u32::from(b) << 7;
  if b < 0x80 {
    return Ok((6, value + (u64::from(part1) << 28)));
  };
  part1 -= 0x80 << 7;
  b = unsafe { *bytes.get_unchecked(6) };
  part1 += u32::from(b) << 14;
  if b < 0x80 {
    return Ok((7, value + (u64::from(part1) << 28)));
  };
  part1 -= 0x80 << 14;
  b = unsafe { *bytes.get_unchecked(7) };
  part1 += u32::from(b) << 21;
  if b < 0x80 {
    return Ok((8, value + (u64::from(part1) << 28)));
  };
  part1 -= 0x80 << 21;
  let value = value + ((u64::from(part1)) << 28);

  b = unsafe { *bytes.get_unchecked(8) };
  let mut part2: u32 = u32::from(b);
  if b < 0x80 {
    return Ok((9, value + (u64::from(part2) << 56)));
  };
  part2 -= 0x80;
  b = unsafe { *bytes.get_unchecked(9) };
  part2 += u32::from(b) << 7;
  // Check for u64::MAX overflow. See [`ConsumeVarint`][1] for details.
  // [1]: https://github.com/protocolbuffers/protobuf-go/blob/v1.27.1/encoding/protowire/wire.go#L358
  if b < 0x02 {
    return Ok((10, value + (u64::from(part2) << 56)));
  };

  // We have overrun the maximum size of a varint (10 bytes) or the final byte caused an overflow.
  // Assume the data is corrupt.
  Err(VarintError::Invalid)
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be between 1 and 10, inclusive.
#[inline]
pub fn encoded_len_varint(value: u64) -> usize {
  // Based on [VarintSize64][1].
  // [1]: https://github.com/google/protobuf/blob/3.3.x/src/google/protobuf/io/coded_stream.h#L1301-L1309
  ((((value | 1).leading_zeros() ^ 63) * 9 + 73) / 64) as usize
}
