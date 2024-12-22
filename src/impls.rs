use super::*;

#[cfg(any(feature = "alloc", feature = "std"))]
mod bytes;

#[cfg(any(feature = "alloc", feature = "std"))]
mod string;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use string::*;

#[cfg(any(feature = "alloc", feature = "std"))]
mod vec;

#[cfg(feature = "std")]
mod net;
#[cfg(feature = "std")]
pub use net::*;

mod time;
pub use time::*;

mod numbers;
pub use numbers::*;

#[cfg(feature = "smallvec1")]
mod smallvec1;

mod bytes_array;

use byteorder::{ByteOrder, NetworkEndian};

trait ToNetworkEndian {
  fn to_network_endian(&self) -> impl AsRef<[u8]>;

  fn from_network_endian(src: &[u8]) -> Self;
}

macro_rules! impl_network_endian {
  ($($ty:ident), + $(,)?) => {
    $(
      impl ToNetworkEndian for $ty {
        fn to_network_endian(&self) -> impl AsRef<[u8]> {
          let mut buf = [0; core::mem::size_of::<$ty>()];
          paste::paste! {
            NetworkEndian::[< write_ $ty >](&mut buf, *self);
          }
          buf
        }

        fn from_network_endian(src: &[u8]) -> Self {
          paste::paste! {
            NetworkEndian::[< read_ $ty >](src)
          }
        }
      }
    )*
  };
}

impl ToNetworkEndian for u8 {
  fn to_network_endian(&self) -> impl AsRef<[u8]> {
    [*self]
  }

  fn from_network_endian(src: &[u8]) -> Self {
    src[0]
  }
}

impl ToNetworkEndian for i8 {
  fn to_network_endian(&self) -> impl AsRef<[u8]> {
    [*self as u8]
  }

  fn from_network_endian(src: &[u8]) -> Self {
    src[0] as i8
  }
}

impl_network_endian!(u16, u32, u64, u128, i16, i32, i64, i128,);

/// The error type for errors that get returned when encoding or decoding fails.
#[derive(Debug, thiserror::Error)]
pub enum BytesTransformError {
  /// Returned when the buffer is too small to encode.
  #[error(
    "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
  )]
  EncodeBufferTooSmall,
  /// Returned when the bytes are corrupted.
  #[error("not enough bytes to decode")]
  NotEnoughBytes,
}

#[cfg(all(feature = "std", feature = "async"))]
async fn decode_bytes_from_async<R: futures_util::io::AsyncRead + Unpin>(
  src: &mut R,
) -> std::io::Result<(usize, Vec<u8>)> {
  use futures_util::io::AsyncReadExt;

  let mut len_buf = [0u8; MESSAGE_SIZE_LEN];
  src.read_exact(&mut len_buf).await?;
  let len = u32::from_network_endian(&len_buf) as usize;
  let mut buf = vec![0u8; len];
  src
    .read_exact(&mut buf)
    .await
    .map(|_| (len + MESSAGE_SIZE_LEN, buf))
}

#[cfg(feature = "std")]
fn decode_bytes_from<R: std::io::Read>(src: &mut R) -> std::io::Result<(usize, Vec<u8>)> {
  let mut len_buf = [0u8; MESSAGE_SIZE_LEN];
  src.read_exact(&mut len_buf)?;
  let len = u32::from_network_endian(&len_buf) as usize;
  let mut buf = vec![0u8; len];
  src
    .read_exact(&mut buf)
    .map(|_| (MESSAGE_SIZE_LEN + len, buf))
}

#[cfg(any(feature = "alloc", feature = "std"))]
fn decode_bytes(src: &[u8]) -> Result<(usize, ::std::vec::Vec<u8>), ()> {
  let len = src.len();
  if len < MESSAGE_SIZE_LEN {
    return Err(());
  }

  let data_len = u32::from_network_endian(&src[..MESSAGE_SIZE_LEN]) as usize;
  if data_len > len - MESSAGE_SIZE_LEN {
    return Err(());
  }

  let total_len = MESSAGE_SIZE_LEN + data_len;
  Ok((
    total_len,
    src[MESSAGE_SIZE_LEN..MESSAGE_SIZE_LEN + data_len].to_vec(),
  ))
}

#[cfg(any(feature = "alloc", feature = "std"))]
fn encode_bytes(src: &[u8], dst: &mut [u8]) -> Result<usize, ()> {
  let encoded_len = encoded_bytes_len(src);
  if dst.len() < encoded_len {
    return Err(());
  }
  let src_len = src.len();
  NetworkEndian::write_u32(&mut dst[..MESSAGE_SIZE_LEN], src_len as u32);
  dst[MESSAGE_SIZE_LEN..MESSAGE_SIZE_LEN + src_len].copy_from_slice(src);
  Ok(MESSAGE_SIZE_LEN + src_len)
}

#[cfg(feature = "std")]
fn encode_bytes_to<W: std::io::Write>(src: &[u8], dst: &mut W) -> std::io::Result<usize> {
  let len = src.len();
  if len + MESSAGE_SIZE_LEN <= MAX_INLINED_BYTES {
    let mut buf = [0u8; MAX_INLINED_BYTES];
    NetworkEndian::write_u32(&mut buf[..MESSAGE_SIZE_LEN], len as u32);
    buf[MESSAGE_SIZE_LEN..MESSAGE_SIZE_LEN + len].copy_from_slice(src);
    dst
      .write_all(&buf[..MESSAGE_SIZE_LEN + len])
      .map(|_| MESSAGE_SIZE_LEN + len)
  } else {
    let mut buf = std::vec![0; MESSAGE_SIZE_LEN + len];
    NetworkEndian::write_u32(&mut buf[..MESSAGE_SIZE_LEN], len as u32);
    buf[MESSAGE_SIZE_LEN..].copy_from_slice(src);
    dst.write_all(&buf).map(|_| MESSAGE_SIZE_LEN + len)
  }
}

#[cfg(all(feature = "std", feature = "async"))]
async fn encode_bytes_to_async<W: futures_util::io::AsyncWrite + Unpin>(
  src: &[u8],
  dst: &mut W,
) -> std::io::Result<usize> {
  use futures_util::io::AsyncWriteExt;

  let len = src.len();
  if len + MESSAGE_SIZE_LEN <= MAX_INLINED_BYTES {
    let mut buf = [0u8; MAX_INLINED_BYTES];
    NetworkEndian::write_u32(&mut buf[..MESSAGE_SIZE_LEN], len as u32);
    buf[MESSAGE_SIZE_LEN..MESSAGE_SIZE_LEN + len].copy_from_slice(src);
    dst
      .write_all(&buf[..MESSAGE_SIZE_LEN + len])
      .await
      .map(|_| MESSAGE_SIZE_LEN + len)
  } else {
    let mut buf = std::vec![0; MESSAGE_SIZE_LEN + len];
    NetworkEndian::write_u32(&mut buf[..MESSAGE_SIZE_LEN], len as u32);
    buf[MESSAGE_SIZE_LEN..].copy_from_slice(src);
    dst.write_all(&buf).await.map(|_| MESSAGE_SIZE_LEN + len)
  }
}

#[cfg(any(feature = "alloc", feature = "std"))]
fn encoded_bytes_len(src: &[u8]) -> usize {
  MESSAGE_SIZE_LEN + src.len()
}
