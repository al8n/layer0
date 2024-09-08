/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be between 1 and 10, inclusive.
#[inline]
pub const fn encoded_len_varint(value: u64) -> usize {
  // Based on [VarintSize64][1].
  // [1]: https://github.com/google/protobuf/blob/3.3.x/src/google/protobuf/io/coded_stream.h#L1301-L1309
  ((((value | 1).leading_zeros() ^ 63) * 9 + 73) / 64) as usize
}

/// Encoding varint error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncodeVarintError {
  /// The buffer did not have enough space to encode the value.
  BufferTooSmall,
}

impl core::fmt::Display for EncodeVarintError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::BufferTooSmall => write!(
        f,
        "the buffer did not have enough space to encode the value"
      ),
    }
  }
}

#[cfg(feature = "std")]
impl std::error::Error for EncodeVarintError {}

/// Encodes an integer value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub fn encode_varint(mut x: u64, buf: &mut [u8]) -> Result<usize, EncodeVarintError> {
  let mut i = 0;

  while x >= 0x80 {
    if i >= buf.len() {
      return Err(EncodeVarintError::BufferTooSmall);
    }

    buf[i] = (x as u8) | 0x80;
    x >>= 7;
    i += 1;
  }
  buf[i] = x as u8;
  Ok(i + 1)
}

/// Decoding varint error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecodeVarintError {
  /// The buffer did not contain a valid LEB128 encoding.
  Overflow,
  /// The buffer did not contain enough bytes to decode a value.
  NotEnoughBytes,
}

impl core::fmt::Display for DecodeVarintError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Overflow => write!(f, "overflow"),
      Self::NotEnoughBytes => write!(
        f,
        "the buffer did not contain enough bytes to decode a value"
      ),
    }
  }
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeVarintError {}

/// Decodes a value from LEB128 variable length format.
///
/// # Arguments
///
/// * `buf` - A byte slice containing the LEB128 encoded value.
///
/// # Returns
///
/// * Returns the bytes readed and the decoded value as `u64` if successful.
///
/// * Returns [`DecodeVarintError`] if the buffer did not contain a valid LEB128 encoding
///   or the decode buffer did not contain enough bytes to decode a value.
pub const fn decode_varint(buf: &[u8]) -> Result<(usize, u64), DecodeVarintError> {
  let (mut x, mut s) = (0, 0);
  let mut i = 0usize;
  loop {
    if i == 10 {
      // It's not a valid LEB128 encoding if it exceeds 10 bytes for u64.
      return Err(DecodeVarintError::Overflow);
    }

    if i >= buf.len() {
      return Err(DecodeVarintError::NotEnoughBytes);
    }

    let b = buf[i];
    if b < 0x80 {
      if i == 10 - 1 && b > 1 {
        return Err(DecodeVarintError::Overflow);
      }
      return Ok((i + 1, x | (b as u64) << s));
    }
    x |= ((b & 0x7f) as u64) << s;
    s += 7;
    i += 1;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn check(value: u64, encoded: &[u8]) {
    let mut expected = [0u8; 16];

    let a = encode_varint(value, &mut expected).unwrap();
    assert_eq!(&expected[..a], encoded);
    assert_eq!(a, encoded.len());

    let roundtrip = decode_varint(&expected[..a]).unwrap();
    assert_eq!(roundtrip.1, value);
    assert_eq!(roundtrip.0, encoded.len());
  }

  #[test]
  fn roundtrip_u64() {
    check(2u64.pow(0) - 1, &[0x00]);
    check(2u64.pow(0), &[0x01]);

    check(2u64.pow(7) - 1, &[0x7F]);
    check(2u64.pow(7), &[0x80, 0x01]);
    check(300u64, &[0xAC, 0x02]);

    check(2u64.pow(14) - 1, &[0xFF, 0x7F]);
    check(2u64.pow(14), &[0x80, 0x80, 0x01]);

    check(2u64.pow(21) - 1, &[0xFF, 0xFF, 0x7F]);
    check(2u64.pow(21), &[0x80, 0x80, 0x80, 0x01]);

    check(2u64.pow(28) - 1, &[0xFF, 0xFF, 0xFF, 0x7F]);
    check(2u64.pow(28), &[0x80, 0x80, 0x80, 0x80, 0x01]);

    check(2u64.pow(35) - 1, &[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
    check(2u64.pow(35), &[0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);

    check(2u64.pow(42) - 1, &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
    check(2u64.pow(42), &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);

    check(
      2u64.pow(49) - 1,
      &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
    );
    check(
      2u64.pow(49),
      &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
    );

    check(
      2u64.pow(56) - 1,
      &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
    );
    check(
      2u64.pow(56),
      &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
    );

    check(
      2u64.pow(63) - 1,
      &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
    );
    check(
      2u64.pow(63),
      &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
    );

    check(
      u64::MAX,
      &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
    );
  }

  #[test]
  fn test_large_number_encode_decode() {
    let mut buffer = [0u8; 10];
    let original = 30000u64;
    let encoded_len = encode_varint(original, &mut buffer).unwrap();
    let (bytes_read, decoded) = decode_varint(&buffer).unwrap();
    assert_eq!(original, decoded);
    assert_eq!(bytes_read, encoded_len);
  }

  #[test]
  fn test_buffer_too_small_error() {
    let mut buffer = [0u8; 1]; // Intentionally small buffer
    match encode_varint(u64::MAX, &mut buffer) {
      Err(EncodeVarintError::BufferTooSmall) => (),
      _ => panic!("Expected BufferTooSmall error"),
    }
  }

  #[test]
  fn test_decode_overflow_error() {
    let buffer = [0x80u8; 11]; // More than 10 bytes
    match decode_varint(&buffer) {
      Err(DecodeVarintError::Overflow) => (),
      _ => panic!("Expected Overflow error"),
    }
  }
}
