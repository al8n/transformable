use core::{mem, time::Duration};

use super::Transformable;

const ENCODED_LEN: usize = mem::size_of::<u64>() + mem::size_of::<u32>();

/// Error returned by [`Duration`] when transforming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurationTransformError {
  /// The buffer is too small to encode the value.
  EncodeBufferTooSmall,
  /// NotEnoughBytes binary data.
  NotEnoughBytes,
}

impl core::fmt::Display for DurationTransformError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::EncodeBufferTooSmall => write!(
        f,
        "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
      ),
      Self::NotEnoughBytes => write!(f, "not enough bytes to decode"),
    }
  }
}

impl core::error::Error for DurationTransformError {}

impl Transformable for Duration {
  type Error = DurationTransformError;

  fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error> {
    if dst.len() < self.encoded_len() {
      return Err(Self::Error::EncodeBufferTooSmall);
    }

    let buf = encode_duration_unchecked(*self);
    dst[..ENCODED_LEN].copy_from_slice(&buf);
    Ok(ENCODED_LEN)
  }

  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
    let buf = encode_duration_unchecked(*self);
    let len = buf.len();
    writer.write_all(&buf).map(|_| len)
  }

  #[cfg(feature = "async")]
  #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
  async fn encode_to_async_writer<W: futures_util::io::AsyncWrite + Send + Unpin>(
    &self,
    writer: &mut W,
  ) -> std::io::Result<usize>
  where
    Self::Error: Send + Sync + 'static,
  {
    use futures_util::io::AsyncWriteExt;

    let buf = encode_duration_unchecked(*self);
    let len = buf.len();
    writer.write_all(&buf).await.map(|_| len)
  }

  fn encoded_len(&self) -> usize {
    ENCODED_LEN
  }

  fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
  where
    Self: Sized,
  {
    if src.len() < ENCODED_LEN {
      return Err(Self::Error::NotEnoughBytes);
    }

    Ok(decode_duration_unchecked(src))
  }

  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    let mut buf = [0; ENCODED_LEN];
    reader.read_exact(&mut buf)?;
    Ok(decode_duration_unchecked(&buf))
  }

  #[cfg(feature = "async")]
  #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
  async fn decode_from_async_reader<R: futures_util::io::AsyncRead + Send + Unpin>(
    reader: &mut R,
  ) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
    Self::Error: Send + Sync + 'static,
  {
    use futures_util::AsyncReadExt;

    let mut buf = [0; ENCODED_LEN];
    reader.read_exact(&mut buf).await?;
    Ok(decode_duration_unchecked(&buf))
  }
}

#[inline]
const fn encode_duration_unchecked(dur: Duration) -> [u8; ENCODED_LEN] {
  let secs = dur.as_secs().to_be_bytes();
  let nanos = dur.subsec_nanos().to_be_bytes();
  [
    secs[0], secs[1], secs[2], secs[3], secs[4], secs[5], secs[6], secs[7], nanos[0], nanos[1],
    nanos[2], nanos[3],
  ]
}

#[inline]
const fn decode_duration_unchecked(src: &[u8]) -> (usize, Duration) {
  let secs = u64::from_be_bytes([
    src[0], src[1], src[2], src[3], src[4], src[5], src[6], src[7],
  ]);
  let nanos = u32::from_be_bytes([src[8], src[9], src[10], src[11]]);
  (ENCODED_LEN, Duration::new(secs, nanos))
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub use system_time::*;
#[cfg(feature = "std")]
mod system_time;

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub use instant::*;
#[cfg(feature = "std")]
mod instant;

test_transformable!(Duration => test_duration_transformable(Duration::new(10, 1080)));
