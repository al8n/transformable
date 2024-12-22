use super::*;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};

mod socket_addr;
pub use socket_addr::*;

mod ip_addr;
pub use ip_addr::*;

const ADDR_V4_ENCODED_SIZE: usize = 4;
const ADDR_V6_ENCODED_SIZE: usize = 16;
const PORT_SIZE: usize = 2;

/// The error type for errors that get returned when encoding or decoding fails.
#[derive(Debug, thiserror::Error)]
pub enum AddrTransformError {
  /// Returned when the buffer is too small to encode.
  #[error(
    "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
  )]
  EncodeBufferTooSmall,
  /// Returned when the bytes are corrupted.
  #[error("not enough bytes to decode")]
  NotEnoughBytes,
}

impl AddrTransformError {
  fn from_bytes_error(err: BytesTransformError) -> Self {
    match err {
      BytesTransformError::EncodeBufferTooSmall => Self::EncodeBufferTooSmall,
      BytesTransformError::NotEnoughBytes => Self::NotEnoughBytes,
    }
  }
}

macro_rules! impl_socket_addr {
  ($ty:ident ($ip:ident, $addr_size: ident)) => {
    impl Transformable for $ty {
      type Error = AddrTransformError;

      fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error> {
        let encoded_len = Transformable::encoded_len(self);
        if dst.len() < encoded_len {
          return Err(Self::Error::EncodeBufferTooSmall);
        }
        dst[..$addr_size].copy_from_slice(&self.ip().octets());
        dst[$addr_size..$addr_size + PORT_SIZE].copy_from_slice(&self.port().to_be_bytes());

        Ok(encoded_len)
      }

      #[cfg(feature = "std")]
      fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        let mut buf = [0u8; $addr_size + PORT_SIZE];
        buf[..$addr_size].copy_from_slice(&self.ip().octets());
        buf[$addr_size..$addr_size + PORT_SIZE].copy_from_slice(&self.port().to_be_bytes());
        writer.write_all(&buf).map(|_| $addr_size + PORT_SIZE)
      }

      #[cfg(feature = "async")]
      async fn encode_to_async_writer<W: futures_util::io::AsyncWrite + Send + Unpin>(
        &self,
        writer: &mut W,
      ) -> std::io::Result<usize> {
        use futures_util::AsyncWriteExt;

        let mut buf = [0u8; $addr_size + PORT_SIZE];
        buf[..$addr_size].copy_from_slice(&self.ip().octets());
        buf[$addr_size..$addr_size + PORT_SIZE].copy_from_slice(&self.port().to_be_bytes());
        writer.write_all(&buf).await.map(|_| $addr_size + PORT_SIZE)
      }

      fn encoded_len(&self) -> usize {
        $addr_size + PORT_SIZE
      }

      fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
      where
        Self: Sized,
      {
        if src.len() < $addr_size + PORT_SIZE {
          return Err(Self::Error::NotEnoughBytes);
        }

        let mut buf = [0u8; $addr_size];
        buf.copy_from_slice(&src[..$addr_size]);
        let ip = $ip::from(buf);
        let mut buf = [0; PORT_SIZE];
        buf.copy_from_slice(&src[$addr_size..$addr_size + PORT_SIZE]);
        let port = u16::from_be_bytes(buf);
        Ok(($addr_size + PORT_SIZE, FromIP::from(ip, port)))
      }

      #[cfg(feature = "std")]
      fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<(usize, Self)>
      where
        Self: Sized,
      {
        let mut buf = [0; $addr_size + PORT_SIZE];
        reader.read_exact(&mut buf)?;
        let mut ip_buf = [0; $addr_size];
        ip_buf.copy_from_slice(&buf[..$addr_size]);
        let ip = $ip::from(ip_buf);
        let port = u16::from_be_bytes([buf[$addr_size], buf[$addr_size + 1]]);

        Ok(($addr_size + PORT_SIZE, FromIP::from(ip, port)))
      }

      #[cfg(feature = "async")]
      async fn decode_from_async_reader<R: futures_util::io::AsyncRead + Send + Unpin>(
        reader: &mut R,
      ) -> std::io::Result<(usize, Self)>
      where
        Self: Sized,
      {
        use futures_util::AsyncReadExt;

        let mut buf = [0; $addr_size + PORT_SIZE];
        reader.read_exact(&mut buf).await?;
        let mut ip_buf = [0; $addr_size];
        ip_buf.copy_from_slice(&buf[..$addr_size]);
        let ip = $ip::from(ip_buf);
        let port = u16::from_be_bytes([buf[$addr_size], buf[$addr_size + 1]]);

        Ok(($addr_size + PORT_SIZE, FromIP::from(ip, port)))
      }
    }
  };
}

trait FromIP {
  type Ip;

  fn from(ip: Self::Ip, port: u16) -> Self;
}

impl FromIP for SocketAddrV4 {
  type Ip = Ipv4Addr;

  fn from(ip: Self::Ip, port: u16) -> Self {
    Self::new(ip, port)
  }
}

impl FromIP for SocketAddrV6 {
  type Ip = Ipv6Addr;

  fn from(ip: Self::Ip, port: u16) -> Self {
    Self::new(ip, port, 0, 0)
  }
}

impl_socket_addr!(SocketAddrV4(Ipv4Addr, ADDR_V4_ENCODED_SIZE));
impl_socket_addr!(SocketAddrV6(Ipv6Addr, ADDR_V6_ENCODED_SIZE));

macro_rules! impl_addr {
  ($ty:ident($addr_size:ident)) => {
    impl Transformable for $ty {
      type Error = AddrTransformError;

      fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error> {
        Transformable::encode(&self.octets(), dst).map_err(Self::Error::from_bytes_error)
      }

      #[cfg(feature = "std")]
      fn encode_to_writer<W: std::io::Write>(&self, dst: &mut W) -> std::io::Result<usize> {
        dst.write_all(&self.octets()).map(|_| $addr_size)
      }

      #[cfg(feature = "async")]
      async fn encode_to_async_writer<W: futures_util::io::AsyncWrite + Send + Unpin>(
        &self,
        dst: &mut W,
      ) -> std::io::Result<usize> {
        use futures_util::io::AsyncWriteExt;

        dst.write_all(&self.octets()).await.map(|_| $addr_size)
      }

      fn encoded_len(&self) -> usize {
        $addr_size
      }

      fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
      where
        Self: Sized,
      {
        let (len, octets) = <[u8; $addr_size] as Transformable>::decode(src)
          .map_err(Self::Error::from_bytes_error)?;
        Ok((len, Self::from(octets)))
      }

      #[cfg(feature = "std")]
      fn decode_from_reader<R: std::io::Read>(src: &mut R) -> std::io::Result<(usize, Self)>
      where
        Self: Sized,
      {
        <[u8; $addr_size] as Transformable>::decode_from_reader(src)
          .map(|(len, octets)| (len, Self::from(octets)))
      }

      #[cfg(feature = "async")]
      async fn decode_from_async_reader<R: futures_util::io::AsyncRead + Send + Unpin>(
        src: &mut R,
      ) -> std::io::Result<(usize, Self)>
      where
        Self: Sized,
      {
        <[u8; $addr_size] as Transformable>::decode_from_async_reader(src)
          .await
          .map(|(len, octets)| (len, Self::from(octets)))
      }
    }
  };
}

impl_addr!(Ipv4Addr(ADDR_V4_ENCODED_SIZE));
impl_addr!(Ipv6Addr(ADDR_V6_ENCODED_SIZE));

test_transformable!(SocketAddrV4 => test_socket_addr_v4_transformable(
  SocketAddrV4::new(
    Ipv4Addr::new(127, 0, 0, 1),
    8080
  )
));

test_transformable!(SocketAddrV6 => test_socket_addr_v6_transformable(
  SocketAddrV6::new(
    Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
    8080,
    0,
    0
  ))
);
