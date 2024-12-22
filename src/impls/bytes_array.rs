use super::*;

impl<const N: usize> Transformable for [u8; N] {
  type Error = BytesTransformError;

  fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error> {
    if dst.len() < N {
      return Err(BytesTransformError::EncodeBufferTooSmall);
    }

    dst[..N].copy_from_slice(self);
    Ok(N)
  }

  #[cfg(feature = "std")]
  fn encode_to_writer<W: std::io::Write>(&self, dst: &mut W) -> std::io::Result<usize> {
    dst.write_all(self).map(|_| N)
  }

  #[cfg(feature = "async")]
  async fn encode_to_async_writer<W: futures_util::io::AsyncWrite + Send + Unpin>(
    &self,
    dst: &mut W,
  ) -> std::io::Result<usize> {
    use futures_util::io::AsyncWriteExt;

    dst.write_all(self).await.map(|_| N)
  }

  fn encoded_len(&self) -> usize {
    N
  }

  fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
  where
    Self: Sized,
  {
    let len = src.len();
    if len < N {
      return Err(BytesTransformError::NotEnoughBytes);
    }

    let mut buf = [0; N];
    buf.copy_from_slice(&src[..N]);

    Ok((N, buf))
  }

  #[cfg(feature = "std")]
  fn decode_from_reader<R: std::io::Read>(src: &mut R) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    let mut buf = [0u8; N];
    src.read_exact(&mut buf).map(|_| (N, buf))
  }

  #[cfg(feature = "async")]
  async fn decode_from_async_reader<R: futures_util::io::AsyncRead + Send + Unpin>(
    src: &mut R,
  ) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    use futures_util::io::AsyncReadExt;

    let mut buf = [0u8; N];
    src.read_exact(&mut buf).await.map(|_| (N, buf))
  }
}
