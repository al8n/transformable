use super::*;

#[cfg(not(feature = "std"))]
use ::std::{boxed::Box, string::String};

use ::std::sync::Arc;

use core::borrow::Borrow;

/// The error type for errors that get returned when encoding or decoding str based structs fails.
#[derive(Debug, thiserror::Error)]
pub enum StringTransformError {
  /// Returned when the buffer is too small to encode.
  #[error(
    "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
  )]
  EncodeBufferTooSmall,
  /// Returned when the decoding meet corruption.
  #[error("not enough bytes to decode")]
  NotEnoughBytes,
  /// Returned when the decoding meet utf8 error.
  #[error(transparent)]
  Utf8Error(#[from] core::str::Utf8Error),
}

macro_rules! impl_string {
  ($ty: ty => $test_fn:ident($init: expr)) => {
    impl Transformable for $ty {
      type Error = StringTransformError;

      fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error> {
        let src: &str = self.borrow();
        encode_bytes(src.as_bytes(), dst).map_err(|_| Self::Error::EncodeBufferTooSmall)
      }

      #[cfg(feature = "std")]
      fn encode_to_writer<W: std::io::Write>(&self, dst: &mut W) -> std::io::Result<usize> {
        let src: &str = self.borrow();
        encode_bytes_to(src.as_bytes(), dst)
      }

      #[cfg(feature = "async")]
      async fn encode_to_async_writer<W: futures_util::io::AsyncWrite + Send + Unpin>(
        &self,
        dst: &mut W,
      ) -> std::io::Result<usize> {
        let src: &str = self.borrow();
        encode_bytes_to_async(src.as_bytes(), dst).await
      }

      fn encoded_len(&self) -> usize {
        let src: &str = self.borrow();
        encoded_bytes_len(src.as_bytes())
      }

      fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
      where
        Self: Sized,
      {
        decode_bytes(src)
          .map_err(|_| Self::Error::NotEnoughBytes)
          .and_then(|(readed, bytes)| {
            core::str::from_utf8(bytes.as_ref())
              .map(|s| (readed, Self::from(s)))
              .map_err(Into::into)
          })
      }

      #[cfg(feature = "std")]
      fn decode_from_reader<R: std::io::Read>(src: &mut R) -> std::io::Result<(usize, Self)>
      where
        Self: Sized,
      {
        decode_bytes_from(src).and_then(|(readed, bytes)| {
          core::str::from_utf8(bytes.as_ref())
            .map(|s| (readed, Self::from(s)))
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
        })
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
          .and_then(|(readed, bytes)| {
            core::str::from_utf8(bytes.as_ref())
              .map(|s| (readed, Self::from(s)))
              .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
          })
      }
    }

    test_transformable!($ty => $test_fn($init));
  };
}

impl_string!(String => test_string_transformable(String::from("hello world")));

#[cfg(feature = "smol_str03")]
impl_string!(smol_str03::SmolStr => test_smol_str03_transformable(smol_str03::SmolStr::from("hello world")));

#[cfg(feature = "smol_str02")]
impl_string!(smol_str02::SmolStr => test_smol_str02_transformable(smol_str02::SmolStr::from("hello world")));

impl_string!(Box<str> => test_box_str_transformable(Box::from("hello world")));

impl_string!(Arc<str> => test_arc_str_transformable(Arc::from("hello world")));
