use super::*;

/// Error type for transformable numbers.
#[derive(Debug)]
pub enum NumberTransformError {
  /// Returned when the buffer is too small to encode.
  EncodeBufferTooSmall,
  /// Returned when there is not enough bytes to decode.
  NotEnoughBytes,
}

impl core::fmt::Display for NumberTransformError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::EncodeBufferTooSmall => write!(f, "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"),
      Self::NotEnoughBytes => write!(f, "not enough bytes to decode"),
    }
  }
}

impl core::error::Error for NumberTransformError {}

macro_rules! impl_number_based_id {
  ($($ty: ty), + $(,)?) => {
    $(
      impl Transformable for $ty {
        type Error = NumberTransformError;

        fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error> {
          const SIZE: usize = core::mem::size_of::<$ty>();

          let encoded_len = Transformable::encoded_len(self);
          if dst.len() < encoded_len {
            return Err(Self::Error::EncodeBufferTooSmall);
          }

          dst[..SIZE].copy_from_slice(self.to_network_endian().as_ref());

          Ok(SIZE)
        }

        #[cfg(feature = "std")]
        fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
          writer.write_all(self.to_network_endian().as_ref()).map(|_| core::mem::size_of::<$ty>())
        }

        #[cfg(feature = "async")]
        async fn encode_to_async_writer<W: futures_util::io::AsyncWrite + Send + Unpin>(
          &self,
          writer: &mut W,
        ) -> std::io::Result<usize> {
          use futures_util::AsyncWriteExt;

          writer.write_all(self.to_network_endian().as_ref()).await.map(|_| core::mem::size_of::<$ty>())
        }

        fn encoded_len(&self) -> usize {
          core::mem::size_of::<$ty>()
        }

        fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error> where Self: Sized {
          const SIZE: usize = core::mem::size_of::<$ty>();

          if src.len() < SIZE {
            return Err(Self::Error::NotEnoughBytes);
          }

          let id = <$ty>::from_network_endian(&src[..SIZE]);
          Ok((SIZE, id))
        }

        #[cfg(feature = "std")]
        fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<(usize, Self)> where Self: Sized {
          const SIZE: usize = core::mem::size_of::<$ty>();

          let mut buf = [0u8; SIZE];
          reader.read_exact(&mut buf)?;
          let id = <$ty>::from_network_endian(&buf);
          Ok((SIZE, id))
        }

        #[cfg(feature = "async")]
        async fn decode_from_async_reader<R: futures_util::io::AsyncRead + Send + Unpin>(
          reader: &mut R,
        ) -> std::io::Result<(usize, Self)>
        where
          Self: Sized,
        {
          use futures_util::AsyncReadExt;

          const SIZE: usize = core::mem::size_of::<$ty>();

          let mut buf = [0u8; SIZE];
          reader.read_exact(&mut buf).await?;
          let id = <$ty>::from_network_endian(&buf);
          Ok((SIZE, id))
        }
      }

      #[cfg(test)]
      paste::paste! {
        test_transformable!($ty => [< test _ $ty _ transformable >](rand::random()));
      }
    )+
  };
}

impl_number_based_id!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128,);
