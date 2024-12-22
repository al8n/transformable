use super::*;
#[cfg(not(feature = "std"))]
use ::std::boxed::Box;
use ::std::sync::Arc;

macro_rules! impl_bytes {
  ($ty: ty => $test_fn:ident($init: expr)) => {
    impl Transformable for $ty {
      type Error = BytesTransformError;

      fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error> {
        encode_bytes(self.as_ref(), dst).map_err(|_| Self::Error::EncodeBufferTooSmall)
      }

      #[cfg(feature = "std")]
      fn encode_to_writer<W: std::io::Write>(&self, dst: &mut W) -> std::io::Result<usize> {
        encode_bytes_to(self.as_ref(), dst)
      }

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
        decode_bytes(src).map(|(readed, b)| (readed, b.into())).map_err(|_| Self::Error::NotEnoughBytes)
      }

      #[cfg(feature = "std")]
      fn decode_from_reader<R: std::io::Read>(src: &mut R) -> std::io::Result<(usize, Self)>
      where
        Self: Sized,
      {
        decode_bytes_from(src).map(|(readed, b)| (readed, b.into()))
      }

      #[cfg(feature = "async")]
      async fn decode_from_async_reader<R: futures_util::io::AsyncRead + Send + Unpin>(
        src: &mut R,
      ) -> std::io::Result<(usize, Self)>
      where
        Self: Sized,
      {
        decode_bytes_from_async(src)
          .await
          .map(|(readed, b)| (readed, b.into()))
      }
    }

    test_transformable!($ty => $test_fn($init));
  };
}

#[cfg(feature = "bytes1")]
impl_bytes!(::bytes1::Bytes => test_bytes_transformable(::bytes1::Bytes::from_static(b"hello world")));
impl_bytes!(Box<[u8]> => test_box_u8_transformable(Box::from(b"hello world".to_vec())));
impl_bytes!(Arc<[u8]> => test_arc_u8_transformable(Arc::from(b"hello world".to_vec())));
