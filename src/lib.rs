#![doc = include_str!("../README.md")]
#![deny(missing_docs, warnings)]
#![forbid(unsafe_code)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

macro_rules! test_transformable {
  ($ty: ty => $test_fn:ident($init: expr)) => {
    #[test]
    fn $test_fn() {
      use crate::TestTransformable;

      <$ty>::test_transformable(|| $init);
    }
  };
}

#[cfg(any(feature = "alloc", feature = "std"))]
const MESSAGE_SIZE_LEN: usize = core::mem::size_of::<u32>();
#[cfg(feature = "std")]
const MAX_INLINED_BYTES: usize = 256;

/// The type can transform its representation between structured and byte form.
pub trait Transformable: Send + Sync + 'static {
  /// The error type returned when encoding or decoding fails.
  type Error: core::error::Error + Send + Sync + 'static;

  /// Encodes the value into the given buffer for transmission.
  ///
  /// Returns the number of bytes written to the buffer.
  fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error>;

  /// Encodes the value into a vec for transmission.
  #[cfg(any(feature = "alloc", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
  fn encode_to_vec(&self) -> Result<::std::vec::Vec<u8>, Self::Error> {
    let mut buf = ::std::vec![0u8; self.encoded_len()];
    self.encode(&mut buf)?;
    Ok(buf)
  }

  /// Encodes the value into the given writer for transmission.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
    let encoded_len = self.encoded_len();
    if encoded_len <= MAX_INLINED_BYTES {
      let mut buf = [0u8; MAX_INLINED_BYTES];
      let len = self.encode(&mut buf).map_err(utils::invalid_data)?;
      writer.write_all(&buf[..encoded_len]).map(|_| len)
    } else {
      let mut buf = ::std::vec![0u8; encoded_len];
      let len = self.encode(&mut buf).map_err(utils::invalid_data)?;
      writer.write_all(&buf).map(|_| len)
    }
  }

  /// Encodes the value into the given async writer for transmission.
  #[cfg(feature = "async")]
  #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
  fn encode_to_async_writer<W: futures_util::io::AsyncWrite + Send + Unpin>(
    &self,
    writer: &mut W,
  ) -> impl std::future::Future<Output = std::io::Result<usize>> + Send {
    use futures_util::io::AsyncWriteExt;
    async move {
      let encoded_len = self.encoded_len();
      if encoded_len <= MAX_INLINED_BYTES {
        let mut buf = [0u8; MAX_INLINED_BYTES];
        let len = self.encode(&mut buf).map_err(utils::invalid_data)?;
        writer.write_all(&buf[..encoded_len]).await.map(|_| len)
      } else {
        let mut buf = ::std::vec![0u8; encoded_len];
        let len = self.encode(&mut buf).map_err(utils::invalid_data)?;
        writer.write_all(&buf).await.map(|_| len)
      }
    }
  }

  /// Returns the encoded length of the value.
  /// This is used to pre-allocate a buffer for encoding.
  fn encoded_len(&self) -> usize;

  /// Decodes the value from the given buffer received over the wire.
  ///
  /// Returns the number of bytes read from the buffer and the struct.
  fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
  where
    Self: Sized;

  /// Decodes the value from the given reader received over the wire.
  ///
  /// Returns the number of bytes read from the reader and the struct.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    use byteorder::{ByteOrder, NetworkEndian};

    let mut len = [0u8; MESSAGE_SIZE_LEN];
    reader.read_exact(&mut len)?;
    let msg_len = NetworkEndian::read_u32(&len) as usize;

    if msg_len <= MAX_INLINED_BYTES {
      let mut buf = [0u8; MAX_INLINED_BYTES];
      buf[..MESSAGE_SIZE_LEN].copy_from_slice(&len);
      reader.read_exact(&mut buf[MESSAGE_SIZE_LEN..msg_len])?;
      Self::decode(&buf[..msg_len]).map_err(utils::invalid_data)
    } else {
      let mut buf = ::std::vec![0u8; msg_len];
      buf[..MESSAGE_SIZE_LEN].copy_from_slice(&len);
      reader.read_exact(&mut buf[MESSAGE_SIZE_LEN..])?;
      Self::decode(&buf).map_err(utils::invalid_data)
    }
  }

  /// Decodes the value from the given async reader received over the wire.
  ///
  /// Returns the number of bytes read from the reader and the struct.
  #[cfg(feature = "async")]
  #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
  fn decode_from_async_reader<R: futures_util::io::AsyncRead + Send + Unpin>(
    reader: &mut R,
  ) -> impl std::future::Future<Output = std::io::Result<(usize, Self)>> + Send
  where
    Self: Sized,
  {
    use byteorder::{ByteOrder, NetworkEndian};
    use futures_util::io::AsyncReadExt;

    async move {
      let mut len = [0u8; MESSAGE_SIZE_LEN];
      reader.read_exact(&mut len).await?;
      let msg_len = NetworkEndian::read_u32(&len) as usize;

      if msg_len <= MAX_INLINED_BYTES {
        let mut buf = [0u8; MAX_INLINED_BYTES];
        buf[..MESSAGE_SIZE_LEN].copy_from_slice(&len);
        reader
          .read_exact(&mut buf[MESSAGE_SIZE_LEN..msg_len])
          .await?;
        Self::decode(&buf[..msg_len]).map_err(utils::invalid_data)
      } else {
        let mut buf = vec![0u8; msg_len];
        buf[..MESSAGE_SIZE_LEN].copy_from_slice(&len);
        reader.read_exact(&mut buf[MESSAGE_SIZE_LEN..]).await?;
        Self::decode(&buf).map_err(utils::invalid_data)
      }
    }
  }
}

/// The type can transform its representation to byte form.
pub trait Encodable: Send + Sync {
  /// The error type returned when encoding or decoding fails.
  type Error: core::error::Error + Send + Sync + 'static;

  /// Encodes the value into the given buffer for transmission.
  ///
  /// Returns the number of bytes written to the buffer.
  fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error>;

  /// Encodes the value into a vec for transmission.
  #[cfg(any(feature = "alloc", feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
  fn encode_to_vec(&self) -> Result<::std::vec::Vec<u8>, Self::Error> {
    let mut buf = ::std::vec![0u8; self.encoded_len()];
    self.encode(&mut buf)?;
    Ok(buf)
  }

  /// Encodes the value into the given writer for transmission.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
    let encoded_len = self.encoded_len();
    if encoded_len <= MAX_INLINED_BYTES {
      let mut buf = [0u8; MAX_INLINED_BYTES];
      let len = self.encode(&mut buf).map_err(utils::invalid_data)?;
      writer.write_all(&buf[..encoded_len]).map(|_| len)
    } else {
      let mut buf = ::std::vec![0u8; encoded_len];
      let len = self.encode(&mut buf).map_err(utils::invalid_data)?;
      writer.write_all(&buf).map(|_| len)
    }
  }

  /// Encodes the value into the given async writer for transmission.
  #[cfg(feature = "async")]
  #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
  fn encode_to_async_writer<W: futures_util::io::AsyncWrite + Send + Unpin>(
    &self,
    writer: &mut W,
  ) -> impl std::future::Future<Output = std::io::Result<usize>> + Send {
    use futures_util::io::AsyncWriteExt;
    async move {
      let encoded_len = self.encoded_len();
      if encoded_len <= MAX_INLINED_BYTES {
        let mut buf = [0u8; MAX_INLINED_BYTES];
        let len = self.encode(&mut buf).map_err(utils::invalid_data)?;
        writer.write_all(&buf[..encoded_len]).await.map(|_| len)
      } else {
        let mut buf = ::std::vec![0u8; encoded_len];
        let len = self.encode(&mut buf).map_err(utils::invalid_data)?;
        writer.write_all(&buf).await.map(|_| len)
      }
    }
  }

  /// Returns the encoded length of the value.
  /// This is used to pre-allocate a buffer for encoding.
  fn encoded_len(&self) -> usize;
}

impl<T: Transformable> Encodable for T {
  type Error = T::Error;

  fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error> {
    Transformable::encode(self, dst)
  }

  fn encoded_len(&self) -> usize {
    Transformable::encoded_len(self)
  }

  #[cfg(any(feature = "alloc", feature = "std"))]
  fn encode_to_vec(&self) -> Result<::std::vec::Vec<u8>, Self::Error> {
    Transformable::encode_to_vec(self)
  }

  #[cfg(feature = "std")]
  fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
    Transformable::encode_to_writer(self, writer)
  }

  #[cfg(feature = "async")]
  fn encode_to_async_writer<W: futures_util::io::AsyncWrite + Send + Unpin>(
    &self,
    writer: &mut W,
  ) -> impl std::future::Future<Output = std::io::Result<usize>> + Send {
    Transformable::encode_to_async_writer(self, writer)
  }
}

/// The type can transform its representation from byte form to struct.
pub trait Decodable: Send + 'static {
  /// The error type returned when encoding or decoding fails.
  type Error: core::error::Error + Send + Sync + 'static;

  /// Decodes the value from the given buffer received over the wire.
  ///
  /// Returns the number of bytes read from the buffer and the struct.
  fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
  where
    Self: Sized;

  /// Decodes the value from the given reader received over the wire.
  ///
  /// Returns the number of bytes read from the reader and the struct.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<(usize, Self)>
  where
    Self: Sized;

  /// Decodes the value from the given async reader received over the wire.
  ///
  /// Returns the number of bytes read from the reader and the struct.
  #[cfg(feature = "async")]
  #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
  fn decode_from_async_reader<R: futures_util::io::AsyncRead + Send + Unpin>(
    reader: &mut R,
  ) -> impl std::future::Future<Output = std::io::Result<(usize, Self)>> + Send
  where
    Self: Sized;
}

impl<T: Transformable> Decodable for T {
  type Error = T::Error;

  fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error> {
    Transformable::decode(src)
  }

  #[cfg(feature = "std")]
  fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<(usize, Self)>
  where
    Self: Sized,
  {
    Transformable::decode_from_reader(reader)
  }

  #[cfg(feature = "async")]
  fn decode_from_async_reader<R: futures_util::io::AsyncRead + Send + Unpin>(
    reader: &mut R,
  ) -> impl std::future::Future<Output = std::io::Result<(usize, Self)>> + Send
  where
    Self: Sized + Send,
  {
    <Self as Transformable>::decode_from_async_reader::<R>(reader)
  }
}

#[cfg(test)]
trait TestTransformable: Transformable + Eq + core::fmt::Debug + Sized {
  fn test_transformable(init: impl FnOnce() -> Self)
  where
    <Self as Transformable>::Error: core::error::Error + Send + Sync + 'static,
  {
    let val = init();
    let mut buf = std::vec![0; val.encoded_len()];
    val.encode(&mut buf).unwrap();
    let (_, decoded) = Self::decode(&buf).unwrap();
    assert_eq!(decoded, val);

    #[cfg(feature = "std")]
    {
      let mut buf = std::vec::Vec::new();
      val.encode_to_writer(&mut buf).unwrap();
      let (_, decoded) = Self::decode_from_reader(&mut buf.as_slice()).unwrap();
      assert_eq!(decoded, val);
    }
  }
}

#[cfg(test)]
impl<T: Transformable + Eq + core::fmt::Debug + Sized> TestTransformable for T {}

mod impls;
pub use impls::*;

/// Utilities for encoding and decoding.
pub mod utils;
