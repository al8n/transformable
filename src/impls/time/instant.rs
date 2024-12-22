use std::{
  sync::OnceLock,
  time::{Instant, SystemTime, SystemTimeError},
};

use super::*;

/// Error returned by [`Instant`] when transforming.
#[derive(Debug, Clone)]
pub enum InstantTransformError {
  /// The buffer is too small to encode the value.
  EncodeBufferTooSmall,
  /// NotEnoughBytes binary data.
  NotEnoughBytes,
  /// Invalid system time.
  InvalidSystemTime(SystemTimeError),
}

impl core::fmt::Display for InstantTransformError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
    Self::EncodeBufferTooSmall => write!(
      f,
      "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
    ),
    Self::NotEnoughBytes => write!(f, "not enough bytes to decode instant"),
    Self::InvalidSystemTime(e) => write!(f, "{e}"),
  }
  }
}

impl core::error::Error for InstantTransformError {}

impl Transformable for Instant {
  type Error = InstantTransformError;

  fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error> {
    if dst.len() < self.encoded_len() {
      return Err(Self::Error::EncodeBufferTooSmall);
    }

    let buf = encode_duration_unchecked(encode_instant_to_duration(*self));
    dst[..ENCODED_LEN].copy_from_slice(&buf);
    Ok(ENCODED_LEN)
  }

  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
    let mut buf = [0u8; ENCODED_LEN];
    self
      .encode(&mut buf)
      .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    writer.write_all(&buf).map(|_| ENCODED_LEN)
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
    use futures_util::AsyncWriteExt;

    let mut buf = [0u8; ENCODED_LEN];
    self
      .encode(&mut buf)
      .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    writer.write_all(&buf).await.map(|_| ENCODED_LEN)
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

    let (readed, instant) = decode_duration_unchecked(src);
    Ok((readed, decode_instant_from_duration(instant)))
  }

  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    let mut buf = [0; ENCODED_LEN];
    reader.read_exact(&mut buf)?;
    let (readed, instant) = decode_duration_unchecked(&buf);
    Ok((readed, decode_instant_from_duration(instant)))
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
    let (readed, instant) = decode_duration_unchecked(&buf);
    Ok((readed, decode_instant_from_duration(instant)))
  }
}

fn init(now: Instant) -> (SystemTime, Instant) {
  static ONCE: OnceLock<(SystemTime, Instant)> = OnceLock::new();
  *ONCE.get_or_init(|| {
    let system_now = SystemTime::now();
    (system_now, now)
  })
}

#[inline]
fn encode_instant_to_duration(instant: Instant) -> Duration {
  let (system_now, instant_now) = init(instant);
  if instant <= instant_now {
    system_now.duration_since(SystemTime::UNIX_EPOCH).unwrap() + (instant_now - instant)
  } else {
    system_now.duration_since(SystemTime::UNIX_EPOCH).unwrap() + (instant - instant_now)
  }
}

#[inline]
fn decode_instant_from_duration(duration: Duration) -> Instant {
  let (system_now, instant_now) = init(Instant::now());
  let system_time = SystemTime::UNIX_EPOCH + duration;
  if system_time >= system_now {
    instant_now + system_time.duration_since(system_now).unwrap()
  } else {
    instant_now - system_now.duration_since(system_time).unwrap()
  }
}

test_transformable!(Instant => test_instant_transformable({
  let now = Instant::now();
  std::thread::sleep(std::time::Duration::from_millis(10));
  now
}));
