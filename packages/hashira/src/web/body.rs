use crate::{error::Error, types::TryBoxStream};
use bytes::{BufMut, Bytes, BytesMut};
use futures::{StreamExt, TryStreamExt};
use std::{convert::Infallible, fmt::Debug};
use thiserror::Error;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Debug, Error)]
pub enum InvalidBodyError {
    #[error("body is a stream")]
    Stream,
}

/// The actual body contents.
pub enum Payload {
    /// The body bytes.
    Bytes(Bytes),

    /// The body stream.
    Stream(TryBoxStream<Bytes>),
}

/// The body of a request/response.
pub struct Body(Payload);

impl Body {
    /// Creates an empty body.
    pub fn empty() -> Self {
        let bytes = Bytes::new();
        Body(Payload::Bytes(bytes))
    }

    /// Creates a stream and returns a sender to add bytes to the body stream.
    pub fn stream() -> (UnboundedSender<Bytes>, Self) {
        let (tx, rx) = unbounded_channel::<Bytes>();

        let stream = UnboundedReceiverStream::new(rx)
            .map(Ok::<_, Infallible>)
            .map_err(|e| e.into());
        let body_stream = Box::pin(stream);
        (tx, Body(Payload::Stream(body_stream)))
    }

    /// Returns `true` if the body is a stream.
    pub fn is_stream(&self) -> bool {
        matches!(&self.0, Payload::Stream(_))
    }

    /// Returns the inner body.
    pub fn into_inner(self) -> Payload {
        self.0
    }

    /// Returns a references to the bytes of the body if possible.
    pub fn try_as_bytes(&self) -> Result<&Bytes, InvalidBodyError> {
        match &self.0 {
            Payload::Bytes(bytes) => Ok(bytes),
            Payload::Stream(_) => Err(InvalidBodyError::Stream),
        }
    }

    /// Returns a future that resolves to the bytes of this body.
    pub async fn into_bytes(self) -> Result<Bytes, Error> {
        match self.0 {
            Payload::Bytes(bytes) => Ok(bytes),
            Payload::Stream(mut stream) => {
                let mut collector = BytesMut::new();

                while let Some(ret) = stream.next().await {
                    let bytes = ret?;
                    collector.put(bytes);
                }

                Ok(collector.into())
            }
        }
    }

    /// Converts the body into a stream.
    pub fn into_stream(self) -> TryBoxStream<Bytes> {
        match self.0 {
            Payload::Bytes(bytes) => Box::pin(futures::stream::once(async move { Ok(bytes) })),
            Payload::Stream(stream) => stream,
        }
    }
}

impl Default for Body {
    fn default() -> Self {
        Body::empty()
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Payload::Bytes(bytes) => write!(f, "Body(Bytes({:?}))", bytes),
            Payload::Stream(_) => write!(f, "Body(Stream)"),
        }
    }
}

impl From<Bytes> for Body {
    fn from(value: Bytes) -> Self {
        Body(Payload::Bytes(value))
    }
}

impl From<BytesMut> for Body {
    fn from(value: BytesMut) -> Self {
        Body(Payload::Bytes(value.into()))
    }
}

impl From<TryBoxStream<Bytes>> for Body {
    fn from(value: TryBoxStream<Bytes>) -> Self {
        Body(Payload::Stream(value))
    }
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        Bytes::from(value).into()
    }
}

impl From<&'static str> for Body {
    fn from(value: &'static str) -> Self {
        Bytes::from(value).into()
    }
}

impl From<&'static [u8]> for Body {
    fn from(value: &'static [u8]) -> Self {
        Bytes::from_static(value).into()
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Bytes::from(value).into()
    }
}
