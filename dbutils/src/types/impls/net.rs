use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};

use super::{InsufficientBuffer, Type, TypeRef, VacantBuffer};

const SOCKET_V6_ENCODED_LEN: usize = 18;
const SOCKET_V4_ENCODED_LEN: usize = 6;
const IPV6_ENCODED_LEN: usize = 16;
const IPV4_ENCODED_LEN: usize = 4;

impl Type for Ipv4Addr {
  type Ref<'a> = Self;

  type Error = InsufficientBuffer;

  #[inline]
  fn encoded_len(&self) -> usize {
    IPV4_ENCODED_LEN
  }

  #[inline]
  fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    buf.put_slice(self.octets().as_ref())
  }
}

impl TypeRef<'_> for Ipv4Addr {
  #[inline]
  unsafe fn from_slice(buf: &[u8]) -> Self {
    let octets = <[u8; IPV4_ENCODED_LEN]>::from_slice(&buf[..IPV4_ENCODED_LEN]);
    Ipv4Addr::from(octets)
  }
}

impl Type for Ipv6Addr {
  type Ref<'a> = Self;

  type Error = InsufficientBuffer;

  #[inline]
  fn encoded_len(&self) -> usize {
    IPV6_ENCODED_LEN
  }

  #[inline]
  fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    buf.put_slice(self.octets().as_ref())
  }
}

impl TypeRef<'_> for Ipv6Addr {
  #[inline]
  unsafe fn from_slice(buf: &[u8]) -> Self {
    let octets = <[u8; IPV6_ENCODED_LEN]>::from_slice(&buf[..IPV6_ENCODED_LEN]);
    Ipv6Addr::from(octets)
  }
}

impl Type for SocketAddrV4 {
  type Ref<'a> = Self;

  type Error = InsufficientBuffer;

  #[inline]
  fn encoded_len(&self) -> usize {
    SOCKET_V4_ENCODED_LEN
  }

  #[inline]
  fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    buf.put_slice(self.ip().octets().as_ref())?;
    buf.put_u16_le(self.port())?;
    Ok(SOCKET_V4_ENCODED_LEN)
  }
}

impl TypeRef<'_> for SocketAddrV4 {
  #[inline]
  unsafe fn from_slice(buf: &[u8]) -> Self {
    let octets = <[u8; 4]>::from_slice(&buf[..4]);
    let port = u16::from_le_bytes(buf[4..6].try_into().unwrap());
    SocketAddrV4::new(Ipv4Addr::from(octets), port)
  }
}

impl Type for SocketAddrV6 {
  type Ref<'a> = Self;

  type Error = InsufficientBuffer;

  #[inline]
  fn encoded_len(&self) -> usize {
    SOCKET_V6_ENCODED_LEN
  }

  #[inline]
  fn encode_to_buffer(&self, buf: &mut VacantBuffer<'_>) -> Result<usize, Self::Error> {
    buf.put_slice(self.ip().octets().as_ref())?;
    buf.put_u16_le(self.port())?;
    Ok(SOCKET_V6_ENCODED_LEN)
  }
}

impl TypeRef<'_> for SocketAddrV6 {
  #[inline]
  unsafe fn from_slice(buf: &[u8]) -> Self {
    let octets = <[u8; IPV6_ENCODED_LEN]>::from_slice(&buf[..IPV6_ENCODED_LEN]);
    let port = u16::from_le_bytes(
      buf[IPV6_ENCODED_LEN..SOCKET_V6_ENCODED_LEN]
        .try_into()
        .unwrap(),
    );
    SocketAddrV6::new(Ipv6Addr::from(octets), port, 0, 0)
  }
}
