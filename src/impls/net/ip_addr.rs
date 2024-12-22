use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::Transformable;

#[cfg(feature = "std")]
use crate::utils::invalid_data;

/// The wire error type for [`IpAddr`].
#[derive(Debug, thiserror::Error)]
pub enum IpAddrTransformError {
  /// Returned when the buffer is too small to encode the [`IpAddr`].
  #[error(
    "buffer is too small, use `IpAddr::encoded_len` to pre-allocate a buffer with enough space"
  )]
  EncodeBufferTooSmall,
  /// Returned when the address family is unknown.
  #[error("invalid address family: {0}, only IPv4 and IPv6 are supported")]
  UnknownAddressFamily(u8),
  /// Returned when the address is corrupted.
  #[error("{0}")]
  NotEnoughBytes(&'static str),
}

const MIN_ENCODED_LEN: usize = TAG_SIZE + V4_SIZE;
const V6_ENCODED_LEN: usize = TAG_SIZE + V6_SIZE;
const V6_SIZE: usize = 16;
const V4_SIZE: usize = 4;
const TAG_SIZE: usize = 1;

impl Transformable for IpAddr {
  type Error = IpAddrTransformError;

  fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error> {
    let encoded_len = self.encoded_len();
    if dst.len() < encoded_len {
      return Err(Self::Error::EncodeBufferTooSmall);
    }
    dst[0] = match self {
      IpAddr::V4(_) => 4,
      IpAddr::V6(_) => 6,
    };
    match self {
      IpAddr::V4(addr) => {
        dst[1..5].copy_from_slice(&addr.octets());
      }
      IpAddr::V6(addr) => {
        dst[1..17].copy_from_slice(&addr.octets());
      }
    }

    Ok(encoded_len)
  }

  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
    match self {
      IpAddr::V4(addr) => {
        let mut buf = [0u8; 7];
        buf[0] = 4;
        buf[1..5].copy_from_slice(&addr.octets());
        writer.write_all(&buf).map(|_| 7)
      }
      IpAddr::V6(addr) => {
        let mut buf = [0u8; 19];
        buf[0] = 6;
        buf[1..17].copy_from_slice(&addr.octets());
        writer.write_all(&buf).map(|_| 19)
      }
    }
  }

  #[cfg(feature = "async")]
  #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
  async fn encode_to_async_writer<W: futures_util::io::AsyncWrite + Send + Unpin>(
    &self,
    writer: &mut W,
  ) -> std::io::Result<usize> {
    use futures_util::AsyncWriteExt;

    match self {
      IpAddr::V4(addr) => {
        let mut buf = [0u8; 7];
        buf[0] = 4;
        buf[1..5].copy_from_slice(&addr.octets());
        writer.write_all(&buf).await.map(|_| 7)
      }
      IpAddr::V6(addr) => {
        let mut buf = [0u8; 19];
        buf[0] = 6;
        buf[1..17].copy_from_slice(&addr.octets());
        writer.write_all(&buf).await.map(|_| 19)
      }
    }
  }

  fn encoded_len(&self) -> usize {
    1 + match self {
      IpAddr::V4(_) => 4,
      IpAddr::V6(_) => 16,
    } + core::mem::size_of::<u16>()
  }

  fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
  where
    Self: Sized,
  {
    match src[0] {
      4 => {
        if src.len() < 7 {
          return Err(IpAddrTransformError::NotEnoughBytes(
            "not enough bytes to decode socket v4 address",
          ));
        }

        let ip = std::net::Ipv4Addr::new(src[1], src[2], src[3], src[4]);
        Ok((MIN_ENCODED_LEN, IpAddr::from(ip)))
      }
      6 => {
        if src.len() < 19 {
          return Err(IpAddrTransformError::NotEnoughBytes(
            "not enough bytes to decode socket v6 address",
          ));
        }

        let mut buf = [0u8; 16];
        buf.copy_from_slice(&src[1..17]);
        let ip = std::net::Ipv6Addr::from(buf);
        Ok((V6_ENCODED_LEN, IpAddr::from(ip)))
      }
      val => Err(IpAddrTransformError::UnknownAddressFamily(val)),
    }
  }

  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    let mut buf = [0; MIN_ENCODED_LEN];
    reader.read_exact(&mut buf)?;
    match buf[0] {
      4 => {
        let ip = Ipv4Addr::new(buf[1], buf[2], buf[3], buf[4]);
        Ok((MIN_ENCODED_LEN, IpAddr::from(ip)))
      }
      6 => {
        let mut remaining = [0; V6_ENCODED_LEN - MIN_ENCODED_LEN];
        reader.read_exact(&mut remaining)?;
        let mut ipv6 = [0; V6_SIZE];
        ipv6[..MIN_ENCODED_LEN - TAG_SIZE].copy_from_slice(&buf[TAG_SIZE..]);
        ipv6[MIN_ENCODED_LEN - TAG_SIZE..]
          .copy_from_slice(&remaining[..V6_ENCODED_LEN - MIN_ENCODED_LEN]);
        let ip = Ipv6Addr::from(ipv6);
        Ok((V6_ENCODED_LEN, IpAddr::from(ip)))
      }
      val => Err(invalid_data(IpAddrTransformError::UnknownAddressFamily(
        val,
      ))),
    }
  }

  #[cfg(feature = "async")]
  #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
  async fn decode_from_async_reader<R: futures_util::io::AsyncRead + Send + Unpin>(
    reader: &mut R,
  ) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    use futures_util::AsyncReadExt;

    let mut buf = [0; MIN_ENCODED_LEN];
    reader.read_exact(&mut buf).await?;
    match buf[0] {
      4 => {
        let ip = Ipv4Addr::new(buf[1], buf[2], buf[3], buf[4]);
        Ok((MIN_ENCODED_LEN, IpAddr::from(ip)))
      }
      6 => {
        let mut remaining = [0; V6_ENCODED_LEN - MIN_ENCODED_LEN];
        reader.read_exact(&mut remaining).await?;
        let mut ipv6 = [0; V6_SIZE];
        ipv6[..MIN_ENCODED_LEN - TAG_SIZE].copy_from_slice(&buf[TAG_SIZE..]);
        ipv6[MIN_ENCODED_LEN - TAG_SIZE..]
          .copy_from_slice(&remaining[..V6_ENCODED_LEN - MIN_ENCODED_LEN]);
        let ip = Ipv6Addr::from(ipv6);
        Ok((V6_ENCODED_LEN, IpAddr::from(ip)))
      }
      val => Err(invalid_data(IpAddrTransformError::UnknownAddressFamily(
        val,
      ))),
    }
  }
}

test_transformable!(IpAddr => test_socket_addr_v4_transformable(
  IpAddr::V4(
    Ipv4Addr::new(127, 0, 0, 1),
  )
));

test_transformable!(IpAddr => test_socket_addr_v6_transformable(
  IpAddr::V6(
    Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
  )
));
