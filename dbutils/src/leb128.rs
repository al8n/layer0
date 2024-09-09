/**
* This file is modified based on the https://github.com/arnohaase/bytes-varint
*/

const MAX_U128_LEB128: usize = 19;
const MAX_U64_LEB128: usize = 10;
const MAX_U32_LEB128: usize = 5;
const MAX_U16_LEB128: usize = 3;

macro_rules! decode_varint {
  (|$buf:ident| $ty:ident::$max_size:ident) => {{
    let mut result = 0;
    let mut shift = 0;
    let mut index = 0;

    loop {
      if index == $max_size {
        return Err(DecodeVarintError::Overflow);
      }

      if index >= $buf.len() {
        return Err(DecodeVarintError::NotEnoughBytes);
      }

      let next = $buf[index] as $ty;

      let v = $ty::BITS as usize / 7 * 7;
      let has_overflow = if shift < v {
        false
      } else if shift == v {
        next & ((u8::MAX << (::core::mem::size_of::<$ty>() % 7)) as $ty) != 0
      } else {
        true
      };

      if has_overflow {
        return Err(DecodeVarintError::Overflow);
      }

      result += (next & 0x7F) << shift;
      if next & 0x80 == 0 {
        break;
      }
      shift += 7;
      index += 1;
    }
    Ok((index + 1, result))
  }};
}

macro_rules! encode_varint {
  ($buf:ident[$x:ident]) => {{
    let mut i = 0;

    while $x >= 0x80 {
      if i >= $buf.len() {
        return Err(EncodeVarintError::BufferTooSmall);
      }

      $buf[i] = ($x as u8) | 0x80;
      $x >>= 7;
      i += 1;
    }
    $buf[i] = $x as u8;
    Ok(i + 1)
  }};
}

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

/// Encodes an `u128` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub fn encode_u128_varint(mut x: u128, buf: &mut [u8]) -> Result<usize, EncodeVarintError> {
  encode_varint!(buf[x])
}

/// Encodes an `u64` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub fn encode_u64_varint(mut x: u64, buf: &mut [u8]) -> Result<usize, EncodeVarintError> {
  encode_varint!(buf[x])
}

/// Encodes an `u32` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub fn encode_u32_varint(mut x: u32, buf: &mut [u8]) -> Result<usize, EncodeVarintError> {
  encode_varint!(buf[x])
}

/// Encodes an `u16` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub fn encode_u16_varint(mut x: u16, buf: &mut [u8]) -> Result<usize, EncodeVarintError> {
  encode_varint!(buf[x])
}

/// Encodes an `i128` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub fn encode_i128_varint(x: i128, buf: &mut [u8]) -> Result<usize, EncodeVarintError> {
  let x = (x << 1) ^ (x >> 127); // Zig-zag encoding
  encode_u128_varint(x as u128, buf)
}

/// Encodes an `i64` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub fn encode_i64_varint(x: i64, buf: &mut [u8]) -> Result<usize, EncodeVarintError> {
  let x = (x << 1) ^ (x >> 63); // Zig-zag encoding
  encode_u64_varint(x as u64, buf)
}

/// Encodes an `i32` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub fn encode_i32_varint(x: i32, buf: &mut [u8]) -> Result<usize, EncodeVarintError> {
  encode_i64_varint(x as i64, buf)
}

/// Encodes an `i16` value into LEB128 variable length format, and writes it to the buffer.
#[inline]
pub fn encode_i16_varint(x: i16, buf: &mut [u8]) -> Result<usize, EncodeVarintError> {
  encode_i64_varint(x as i64, buf)
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
/// * Returns the bytes readed and the decoded value as `u128` if successful.
///
/// * Returns [`DecodeVarintError`] if the buffer did not contain a valid LEB128 encoding
///   or the decode buffer did not contain enough bytes to decode a value.
pub const fn decode_u128_varint(buf: &[u8]) -> Result<(usize, u128), DecodeVarintError> {
  decode_varint!(|buf| u128::MAX_U128_LEB128)
}

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
pub const fn decode_u64_varint(buf: &[u8]) -> Result<(usize, u64), DecodeVarintError> {
  decode_varint!(|buf| u64::MAX_U64_LEB128)
}

/// Decodes a value from LEB128 variable length format.
///
/// # Arguments
///
/// * `buf` - A byte slice containing the LEB128 encoded value.
///
/// # Returns
///
/// * Returns the bytes readed and the decoded value as `u32` if successful.
///
/// * Returns [`DecodeVarintError`] if the buffer did not contain a valid LEB128 encoding
///   or the decode buffer did not contain enough bytes to decode a value.
pub const fn decode_u32_varint(buf: &[u8]) -> Result<(usize, u32), DecodeVarintError> {
  decode_varint!(|buf| u32::MAX_U32_LEB128)
}

/// Decodes a value from LEB128 variable length format.
///
/// # Arguments
///
/// * `buf` - A byte slice containing the LEB128 encoded value.
///
/// # Returns
///
/// * Returns the bytes readed and the decoded value as `u16` if successful.
///
/// * Returns [`DecodeVarintError`] if the buffer did not contain a valid LEB128 encoding
///   or the decode buffer did not contain enough bytes to decode a value.
pub const fn decode_u16_varint(buf: &[u8]) -> Result<(usize, u16), DecodeVarintError> {
  decode_varint!(|buf| u16::MAX_U16_LEB128)
}

/// Decodes a value from LEB128 variable length format.
///
/// # Arguments
///
/// * `buf` - A byte slice containing the LEB128 encoded value.
///
/// # Returns
///
/// * Returns the bytes readed and the decoded value as `i16` if successful.
///
/// * Returns [`DecodeVarintError`] if the buffer did not contain a valid LEB128 encoding
pub fn decode_i16_varint(buf: &[u8]) -> Result<(usize, i16), DecodeVarintError> {
  let (bytes_read, value) = decode_u16_varint(buf)?;
  let value = ((value >> 1) as i16) ^ { -((value & 1) as i16) }; // Zig-zag decoding
  Ok((bytes_read, value))
}

/// Decodes a value from LEB128 variable length format.
///
/// # Arguments
///
/// * `buf` - A byte slice containing the LEB128 encoded value.
///
/// # Returns
///
/// * Returns the bytes readed and the decoded value as `i32` if successful.
///
/// * Returns [`DecodeVarintError`] if the buffer did not contain a valid LEB128 encoding
pub fn decode_i32_varint(buf: &[u8]) -> Result<(usize, i32), DecodeVarintError> {
  let (bytes_read, value) = decode_u32_varint(buf)?;
  let value = ((value >> 1) as i32) ^ { -((value & 1) as i32) }; // Zig-zag decoding
  Ok((bytes_read, value))
}

/// Decodes a value from LEB128 variable length format.
///
/// # Arguments
///
/// * `buf` - A byte slice containing the LEB128 encoded value.
///
/// # Returns
///
/// * Returns the bytes readed and the decoded value as `i64` if successful.
///
/// * Returns [`DecodeVarintError`] if the buffer did not contain a valid LEB128 encoding
pub fn decode_i64_varint(buf: &[u8]) -> Result<(usize, i64), DecodeVarintError> {
  let (bytes_read, value) = decode_u64_varint(buf)?;
  let value = ((value >> 1) as i64) ^ { -((value & 1) as i64) }; // Zig-zag decoding
  Ok((bytes_read, value))
}

/// Decodes a value from LEB128 variable length format.
///
/// # Arguments
///
/// * `buf` - A byte slice containing the LEB128 encoded value.
///
/// # Returns
///
/// * Returns the bytes readed and the decoded value as `i128` if successful.
///
/// * Returns [`DecodeVarintError`] if the buffer did not contain a valid LEB128 encoding
pub fn decode_i128_varint(buf: &[u8]) -> Result<(usize, i128), DecodeVarintError> {
  let (bytes_read, value) = decode_u128_varint(buf)?;
  let value = ((value >> 1) as i128) ^ { -((value & 1) as i128) }; // Zig-zag decoding
  Ok((bytes_read, value))
}

#[cfg(test)]
mod tests {
  use super::*;

  use rstest::*;

  fn check(value: u64, encoded: &[u8]) {
    let mut expected = [0u8; 16];

    let a = encode_u64_varint(value, &mut expected).unwrap();
    assert_eq!(&expected[..a], encoded);
    assert_eq!(a, encoded.len());

    let roundtrip = decode_u64_varint(&expected[..a]).unwrap();
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
    let encoded_len = encode_u64_varint(original, &mut buffer).unwrap();
    let (bytes_read, decoded) = decode_u64_varint(&buffer).unwrap();
    assert_eq!(original, decoded);
    assert_eq!(bytes_read, encoded_len);
  }

  #[test]
  fn test_buffer_too_small_error() {
    let mut buffer = [0u8; 1]; // Intentionally small buffer
    match encode_u64_varint(u64::MAX, &mut buffer) {
      Err(EncodeVarintError::BufferTooSmall) => (),
      _ => panic!("Expected BufferTooSmall error"),
    }
  }

  #[test]
  fn test_decode_overflow_error() {
    let buffer = [0x80u8; 11]; // More than 10 bytes
    match decode_u64_varint(&buffer) {
      Err(DecodeVarintError::Overflow) => (),
      _ => panic!("Expected Overflow error"),
    }

    let buffer = [0x80u8; 6]; // More than 5 bytes
    match decode_u32_varint(&buffer) {
      Err(DecodeVarintError::Overflow) => (),
      _ => panic!("Expected Overflow error"),
    }

    let buffer = [0x80u8; 4]; // More than 3 bytes
    match decode_u16_varint(&buffer) {
      Err(DecodeVarintError::Overflow) => (),
      _ => panic!("Expected Overflow error"),
    }
  }

  // Helper function for zig-zag encoding and decoding
  fn test_zigzag_encode_decode<T>(value: T)
  where
    T: Copy
      + PartialEq
      + std::fmt::Debug
      + std::ops::Shl<Output = T>
      + std::ops::Shr<Output = T>
      + Into<i64>
      + std::convert::TryInto<usize>
      + std::convert::TryFrom<usize>,
  {
    // Encode
    let mut buffer = [0u8; 10];
    let encode_result = encode_i64_varint(value.into(), &mut buffer);
    assert!(encode_result.is_ok(), "Encoding failed");
    let bytes_written = encode_result.unwrap();

    // Decode
    let decode_result = decode_i64_varint(&buffer[..bytes_written]);
    assert!(decode_result.is_ok(), "Decoding failed");
    let (decoded_bytes, decoded_value) = decode_result.unwrap();

    assert_eq!(
      decoded_bytes, bytes_written,
      "Incorrect number of bytes decoded"
    );
    assert_eq!(
      decoded_value,
      value.into(),
      "Decoded value does not match original"
    );
  }

  #[test]
  fn test_zigzag_encode_decode_i16() {
    let values = [-1, 0, 1, -100, 100, i16::MIN, i16::MAX];
    for &value in &values {
      test_zigzag_encode_decode(value);
    }
  }

  #[test]
  fn test_zigzag_encode_decode_i32() {
    let values = [-1, 0, 1, -10000, 10000, i32::MIN, i32::MAX];
    for &value in &values {
      test_zigzag_encode_decode(value);
    }
  }

  #[test]
  fn test_zigzag_encode_decode_i64() {
    let values = [-1, 0, 1, -1000000000, 1000000000, i64::MIN, i64::MAX];
    for &value in &values {
      test_zigzag_encode_decode(value);
    }
  }

  #[rstest]
  #[case::n_0(vec![0], Ok((1, 0)))]
  #[case::n_1(vec![1], Ok((1, 1)))]
  #[case::n_129(vec![0x81, 1], Ok((2, 129)))]
  #[case::max         (vec![0xff, 0xff, 0x03], Ok((3, u16::MAX)))]
  #[case::num_overflow(vec![0xff, 0xff, 0x04], Err(DecodeVarintError::Overflow))]
  #[case::buf_empty(vec![], Err(DecodeVarintError::NotEnoughBytes))]
  #[case::buf_underflow(vec![0x80], Err(DecodeVarintError::NotEnoughBytes))]
  fn test_u16(#[case] bytes: Vec<u8>, #[case] expected: Result<(usize, u16), DecodeVarintError>) {
    let mut bytes = bytes;
    let buf: &[u8] = &mut bytes;
    assert_eq!(expected, decode_u16_varint(buf));

    if let Ok((_, n)) = expected {
      let mut write_buf = vec![0; MAX_U16_LEB128];
      let written = encode_u16_varint(n, &mut write_buf).unwrap();
      assert_eq!(bytes, &write_buf[..written]);
    }
  }

  #[rstest]
  #[case::n_0(vec![0], Ok((1, 0)))]
  #[case::n_1(vec![1], Ok((1, 1)))]
  #[case::n_129(vec![0x81, 1], Ok((2, 129)))]
  #[case::max         (vec![0xff, 0xff, 0xff, 0xff, 0x0f], Ok((5, u32::MAX)))]
  #[case::num_overflow(vec![0xff, 0xff, 0xff, 0xff, 0x10], Err(DecodeVarintError::Overflow))]
  #[case::buf_empty(vec![], Err(DecodeVarintError::NotEnoughBytes))]
  #[case::buf_underflow(vec![0x80], Err(DecodeVarintError::NotEnoughBytes))]
  fn test_u32(#[case] bytes: Vec<u8>, #[case] expected: Result<(usize, u32), DecodeVarintError>) {
    let mut bytes = bytes;
    let buf: &[u8] = &mut bytes;
    assert_eq!(expected, decode_u32_varint(buf));

    if let Ok((_, n)) = expected {
      let mut write_buf = vec![0; MAX_U32_LEB128];
      let written = encode_u32_varint(n, &mut write_buf).unwrap();
      assert_eq!(bytes, &write_buf[..written]);
    }
  }

  #[rstest]
  #[case::n_0(vec![0], Ok((1, 0)))]
  #[case::n_1(vec![1], Ok((1, 1)))]
  #[case::n_129(vec![0x81, 1], Ok((2, 129)))]
  #[case::max         (vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01], Ok((10, u64::MAX)))]
  #[case::num_overflow(vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02], Err(DecodeVarintError::Overflow))]
  #[case::buf_empty(vec![], Err(DecodeVarintError::NotEnoughBytes))]
  #[case::buf_underflow(vec![0x80], Err(DecodeVarintError::NotEnoughBytes))]
  fn test_u64(#[case] bytes: Vec<u8>, #[case] expected: Result<(usize, u64), DecodeVarintError>) {
    let mut bytes = bytes;
    let buf: &[u8] = &mut bytes;
    assert_eq!(expected, decode_u64_varint(buf));

    if let Ok((_, n)) = expected {
      let mut write_buf = vec![0; MAX_U64_LEB128];
      let written = encode_u64_varint(n, &mut write_buf).unwrap();
      assert_eq!(bytes, &write_buf[..written]);
    }
  }

  #[rstest]
  #[case::n_0(vec![0], Ok((1, 0)))]
  #[case::n_1(vec![1], Ok((1, 1)))]
  #[case::n_129(vec![0x81, 1], Ok((2, 129)))]
  #[case::max         (vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x03], Ok((19, u128::MAX)))]
  #[case::num_overflow(vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x04], Err(DecodeVarintError::Overflow))]
  #[case::buf_empty(vec![], Err(DecodeVarintError::NotEnoughBytes))]
  #[case::buf_underflow(vec![0x80], Err(DecodeVarintError::NotEnoughBytes))]
  fn test_u128(#[case] bytes: Vec<u8>, #[case] expected: Result<(usize, u128), DecodeVarintError>) {
    let mut bytes = bytes;
    let buf: &[u8] = &mut bytes;
    assert_eq!(expected, decode_u128_varint(buf));

    if let Ok((_, n)) = expected {
      let mut write_buf = vec![0; MAX_U128_LEB128];
      let written = encode_u128_varint(n, &mut write_buf).unwrap();
      assert_eq!(bytes, &write_buf[..written]);
    }
  }

  #[rstest]
  #[case::n_0(vec![0], Ok((1, 0)))]
  #[case::n_1(vec![2], Ok((1, 1)))]
  #[case::n_128(vec![0x80, 0x2], Ok((2, 128)))]
  #[case::minus_1(vec![0x1], Ok((1, -1)))]
  #[case::minus_129(vec![0x1], Ok((1, -1)))]
  #[case::max               (vec![0xfe, 0xff, 0x03], Ok((3, i16::MAX)))]
  #[case::minus_max         (vec![0xfd, 0xff, 0x03], Ok((3, -i16::MAX)))]
  #[case::min               (vec![0xff, 0xff, 0x03], Ok((3, i16::MIN)))]
  #[case::num_overflow_plus (vec![0xfe, 0xff, 0x04], Err(DecodeVarintError::Overflow))]
  #[case::num_overflow_minus(vec![0xff, 0xff, 0x04], Err(DecodeVarintError::Overflow))]
  #[case::buf_empty(vec![], Err(DecodeVarintError::NotEnoughBytes))]
  #[case::buf_underflow(vec![0x80], Err(DecodeVarintError::NotEnoughBytes))]
  fn test_i16(#[case] bytes: Vec<u8>, #[case] expected: Result<(usize, i16), DecodeVarintError>) {
    let mut bytes = bytes;
    let buf: &[u8] = &mut bytes;
    assert_eq!(expected, decode_i16_varint(buf));

    if let Ok((_, n)) = expected {
      let mut write_buf = vec![0; MAX_U16_LEB128];
      let written = encode_i16_varint(n, &mut write_buf).unwrap();
      assert_eq!(bytes, &write_buf[..written]);
    }
  }

  #[rstest]
  #[case::n_0(vec![0], Ok((1, 0)))]
  #[case::n_1(vec![2], Ok((1, 1)))]
  #[case::n_128(vec![0x80, 0x2], Ok((2, 128)))]
  #[case::minus_1(vec![0x1], Ok((1, -1)))]
  #[case::minus_129(vec![0x1], Ok((1, -1)))]
  #[case::max               (vec![0xfe, 0xff, 0xff, 0xff, 0x0f], Ok((5, i32::MAX)))]
  #[case::minus_max         (vec![0xfd, 0xff, 0xff, 0xff, 0x0f], Ok((5, -i32::MAX)))]
  #[case::min               (vec![0xff, 0xff, 0xff, 0xff, 0x0f], Ok((5, i32::MIN)))]
  #[case::num_overflow_plus (vec![0xfe, 0xff, 0xff, 0xff, 0x10], Err(DecodeVarintError::Overflow))]
  #[case::num_overflow_minus(vec![0xff, 0xff, 0xff, 0xff, 0x10], Err(DecodeVarintError::Overflow))]
  #[case::buf_empty(vec![], Err(DecodeVarintError::NotEnoughBytes))]
  #[case::buf_underflow(vec![0x80], Err(DecodeVarintError::NotEnoughBytes))]
  fn test_i32(#[case] bytes: Vec<u8>, #[case] expected: Result<(usize, i32), DecodeVarintError>) {
    let mut bytes = bytes;
    let buf: &[u8] = &mut bytes;
    assert_eq!(expected, decode_i32_varint(buf));

    if let Ok((_, n)) = expected {
      let mut write_buf = vec![0; MAX_U32_LEB128];
      let written = encode_i32_varint(n, &mut write_buf).unwrap();
      assert_eq!(bytes, &write_buf[..written]);
    }
  }

  #[rstest]
  #[case::n_0(vec![0], Ok((1, 0)))]
  #[case::n_1(vec![2], Ok((1, 1)))]
  #[case::n_128(vec![0x80, 0x2], Ok((2, 128)))]
  #[case::minus_1(vec![0x1], Ok((1, -1)))]
  #[case::minus_129(vec![0x1], Ok((1, -1)))]
  #[case::max               (vec![0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01], Ok((10, i64::MAX)))]
  #[case::minus_max         (vec![0xfd, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01], Ok((10, -i64::MAX)))]
  #[case::min               (vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01], Ok((10, i64::MIN)))]
  #[case::num_overflow_plus (vec![0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02], Err(DecodeVarintError::Overflow))]
  #[case::num_overflow_minus(vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02], Err(DecodeVarintError::Overflow))]
  #[case::buf_empty(vec![], Err(DecodeVarintError::NotEnoughBytes))]
  #[case::buf_underflow(vec![0x80], Err(DecodeVarintError::NotEnoughBytes))]
  fn test_i64(#[case] bytes: Vec<u8>, #[case] expected: Result<(usize, i64), DecodeVarintError>) {
    let mut bytes = bytes;
    let buf: &[u8] = &mut bytes;
    assert_eq!(expected, decode_i64_varint(buf));

    if let Ok((_, n)) = expected {
      let mut write_buf = vec![0; MAX_U64_LEB128];
      let written = encode_i64_varint(n, &mut write_buf).unwrap();
      assert_eq!(bytes, &write_buf[..written]);
    }
  }

  #[rstest]
  #[case::n_0(vec![0], Ok((1, 0)))]
  #[case::n_1(vec![2], Ok((1, 1)))]
  #[case::n_128(vec![0x80, 0x2], Ok((2, 128)))]
  #[case::minus_1(vec![0x1], Ok((1, -1)))]
  #[case::minus_129(vec![0x1], Ok((1, -1)))]
  #[case::max               (vec![0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x03], Ok((19, i128::MAX)))]
  #[case::minus_max         (vec![0xfd, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x03], Ok((19, -i128::MAX)))]
  #[case::min               (vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x03], Ok((19, i128::MIN)))]
  #[case::num_overflow_plus (vec![0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x04], Err(DecodeVarintError::Overflow))]
  #[case::num_overflow_minus(vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x04], Err(DecodeVarintError::Overflow))]
  #[case::buf_empty(vec![], Err(DecodeVarintError::NotEnoughBytes))]
  #[case::buf_underflow(vec![0x80], Err(DecodeVarintError::NotEnoughBytes))]
  fn test_i128(#[case] bytes: Vec<u8>, #[case] expected: Result<(usize, i128), DecodeVarintError>) {
    let mut bytes = bytes;
    let buf: &[u8] = &mut bytes;
    assert_eq!(expected, decode_i128_varint(buf));

    if let Ok((_, n)) = expected {
      let mut write_buf = vec![0; MAX_U128_LEB128];
      let written = encode_i128_varint(n, &mut write_buf).unwrap();
      assert_eq!(bytes, &write_buf[..written]);
    }
  }
}
