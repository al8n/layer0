use core::cmp;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};

use super::{Comparable, InsufficientBuffer, KeyRef, Type, TypeRef, VacantBuffer};

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
  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    let buf_len = buf.len();

    if buf_len < IPV4_ENCODED_LEN {
      return Err(InsufficientBuffer::with_information(
        IPV4_ENCODED_LEN as u64,
        buf_len as u64,
      ));
    }

    buf[..IPV4_ENCODED_LEN].copy_from_slice(self.octets().as_ref());
    Ok(IPV4_ENCODED_LEN)
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

impl KeyRef<'_, Ipv4Addr> for Ipv4Addr {
  #[inline]
  fn compare<Q>(&self, a: &Q) -> cmp::Ordering
  where
    Q: ?Sized + Ord + Comparable<Self>,
  {
    Comparable::compare(a, self).reverse()
  }

  #[inline]
  unsafe fn compare_binary(a: &[u8], b: &[u8]) -> cmp::Ordering {
    unsafe {
      let a = <Self as TypeRef>::from_slice(a);
      let b = <Self as TypeRef>::from_slice(b);
      a.cmp(&b)
    }
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
  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    let buf_len = buf.len();

    if buf_len < IPV6_ENCODED_LEN {
      return Err(InsufficientBuffer::with_information(
        IPV6_ENCODED_LEN as u64,
        buf_len as u64,
      ));
    }

    buf[..IPV6_ENCODED_LEN].copy_from_slice(self.octets().as_ref());
    Ok(IPV6_ENCODED_LEN)
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

impl KeyRef<'_, Ipv6Addr> for Ipv6Addr {
  #[inline]
  fn compare<Q>(&self, a: &Q) -> cmp::Ordering
  where
    Q: ?Sized + Ord + Comparable<Self>,
  {
    Comparable::compare(a, self).reverse()
  }

  #[inline]
  unsafe fn compare_binary(a: &[u8], b: &[u8]) -> cmp::Ordering {
    unsafe {
      let a = <Self as TypeRef>::from_slice(a);
      let b = <Self as TypeRef>::from_slice(b);
      a.cmp(&b)
    }
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
  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    let buf_len = buf.len();

    if buf_len < SOCKET_V4_ENCODED_LEN {
      return Err(InsufficientBuffer::with_information(
        SOCKET_V4_ENCODED_LEN as u64,
        buf_len as u64,
      ));
    }

    buf[..IPV4_ENCODED_LEN].copy_from_slice(self.ip().octets().as_ref());
    buf[IPV4_ENCODED_LEN..SOCKET_V4_ENCODED_LEN].copy_from_slice(&self.port().to_le_bytes());
    Ok(SOCKET_V4_ENCODED_LEN)
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

impl KeyRef<'_, SocketAddrV4> for SocketAddrV4 {
  #[inline]
  fn compare<Q>(&self, a: &Q) -> cmp::Ordering
  where
    Q: ?Sized + Ord + Comparable<Self>,
  {
    Comparable::compare(a, self).reverse()
  }

  #[inline]
  unsafe fn compare_binary(a: &[u8], b: &[u8]) -> cmp::Ordering {
    unsafe {
      let a = <Self as TypeRef>::from_slice(a);
      let b = <Self as TypeRef>::from_slice(b);
      a.cmp(&b)
    }
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
  fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
    let buf_len = buf.len();

    if buf_len < SOCKET_V6_ENCODED_LEN {
      return Err(InsufficientBuffer::with_information(
        SOCKET_V6_ENCODED_LEN as u64,
        buf_len as u64,
      ));
    }

    buf[..IPV6_ENCODED_LEN].copy_from_slice(self.ip().octets().as_ref());
    buf[IPV6_ENCODED_LEN..SOCKET_V6_ENCODED_LEN].copy_from_slice(&self.port().to_le_bytes());
    Ok(SOCKET_V6_ENCODED_LEN)
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

impl KeyRef<'_, SocketAddrV6> for SocketAddrV6 {
  #[inline]
  fn compare<Q>(&self, a: &Q) -> cmp::Ordering
  where
    Q: ?Sized + Ord + Comparable<Self>,
  {
    Comparable::compare(a, self).reverse()
  }

  #[inline]
  unsafe fn compare_binary(a: &[u8], b: &[u8]) -> cmp::Ordering {
    unsafe {
      let a = <Self as TypeRef>::from_slice(a);
      let b = <Self as TypeRef>::from_slice(b);
      a.cmp(&b)
    }
  }
}
