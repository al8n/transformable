use super::*;

impl Transformable for ::std::vec::Vec<u8> {
  type Error = BytesTransformError;

  fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error> {
    encode_bytes(self.as_ref(), dst).map_err(|_| Self::Error::EncodeBufferTooSmall)
  }

  /// Encodes the value into the given writer.
  ///
  /// # Note
  /// The implementation of this method is not optimized, which means
  /// if your writer is expensive (e.g. [`TcpStream`](std::net::TcpStream), [`File`](std::fs::File)),
  /// it is better to use a [`BufWriter`](std::io::BufWriter)
  /// to wrap your orginal writer to cut down the number of I/O times.
  #[cfg(feature = "std")]
  fn encode_to_writer<W: std::io::Write>(&self, dst: &mut W) -> std::io::Result<usize> {
    encode_bytes_to(self.as_ref(), dst)
  }

  /// Encodes the value into the given async writer.
  ///
  /// # Note
  /// The implementation of this method is not optimized, which means
  /// if your writer is expensive (e.g. `TcpStream`, `File`),
  /// it is better to use a [`BufWriter`](futures_util::io::BufWriter)
  /// to wrap your orginal writer to cut down the number of I/O times.
  #[cfg(feature = "async")]
  async fn encode_to_async_writer<W: futures_util::io::AsyncWrite + Send + Unpin>(
    &self,
    dst: &mut W,
  ) -> std::io::Result<usize> {
    encode_bytes_to_async(self.as_ref(), dst).await
  }

  fn encoded_len(&self) -> usize {
    encoded_bytes_len(self.as_ref())
  }

  fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
  where
    Self: Sized,
  {
    decode_bytes(src).map_err(|_| Self::Error::NotEnoughBytes)
  }

  /// Decodes the value from the given reader.
  ///
  /// # Note
  /// The implementation of this method is not optimized, which means
  /// if your reader is expensive (e.g. [`TcpStream`](std::net::TcpStream), [`File`](std::fs::File)),
  /// it is better to use a [`BufReader`](std::io::BufReader)
  /// to wrap your orginal reader to cut down the number of I/O times.
  #[cfg(feature = "std")]
  fn decode_from_reader<R: std::io::Read>(src: &mut R) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    decode_bytes_from(src)
  }

  /// Decodes the value from the given async reader.
  ///
  /// # Note
  /// The implementation of this method is not optimized, which means
  /// if your reader is expensive (e.g. `TcpStream`, `File`),
  /// it is better to use a [`BufReader`](futures_util::io::BufReader)
  /// to wrap your orginal reader to cut down the number of I/O times.
  #[cfg(feature = "async")]
  async fn decode_from_async_reader<R: futures_util::io::AsyncRead + Send + Unpin>(
    src: &mut R,
  ) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    decode_bytes_from_async(src).await
  }
}

test_transformable!(Vec<u8> => test_vec_transformable(std::vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]));
