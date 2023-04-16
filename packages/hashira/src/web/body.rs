use crate::{error::Error, types::TryBoxStream};
use bytes::{BufMut, Bytes, BytesMut};
use futures::{StreamExt, TryStreamExt};
use std::{convert::Infallible, fmt::Debug};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

/// The inner body representation.
pub enum BodyInner {
    /// The body bytes.
    Bytes(Bytes),

    /// The body stream.
    Stream(TryBoxStream<Bytes>),
}

/// The body of a request/response.
pub struct Body(BodyInner);

impl Body {
    /// Creates an empty body.
    pub fn empty() -> Self {
        let bytes = Bytes::new();
        Body(BodyInner::Bytes(bytes))
    }

    /// Creates a stream and returns a sender to add bytes to the body stream.
    pub fn stream() -> (UnboundedSender<Bytes>, Self) {
        let (tx, rx) = unbounded_channel::<Bytes>();

        let stream = UnboundedReceiverStream::new(rx)
            .map(|bytes| Ok::<_, Infallible>(bytes))
            .map_err(|e| e.into());
        let body_stream = Box::pin(stream);
        (tx, Body(BodyInner::Stream(body_stream)))
    }

    /// Returns `true` if the body is a stream.
    pub fn is_stream(&self) -> bool {
        matches!(&self.0, BodyInner::Stream(_))
    }

    /// Returns the inner body.
    pub fn into_inner(self) -> BodyInner {
        self.0
    }

    /// Returns the bytes of this body if possible, fails if the body is a stream.
    pub fn try_into_bytes(self) -> Result<Bytes, Self> {
        match self.0 {
            BodyInner::Bytes(bytes) => Ok(bytes),
            inner => Err(Body(inner)),
        }
    }

    /// Returns the stream of this body, fails if the body is no a stream.
    pub fn try_into_stream(self) -> Result<TryBoxStream<Bytes>, Self> {
        match self.0 {
            BodyInner::Stream(stream) => Ok(stream),
            inner => Err(Body(inner)),
        }
    }

    /// Returns a future that resolves to the bytes of this body.
    pub async fn into_bytes(self) -> Result<Bytes, Error> {
        match self.0 {
            BodyInner::Bytes(bytes) => Ok(bytes),
            BodyInner::Stream(mut stream) => {
                let mut collector = BytesMut::new();

                while let Some(ret) = stream.next().await {
                    let bytes = ret?;
                    collector.put(bytes);
                }

                Ok(collector.into())
            }
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
            BodyInner::Bytes(bytes) => write!(f, "Body(Bytes({:?}))", bytes),
            BodyInner::Stream(_) => write!(f, "Body(Stream)"),
        }
    }
}

impl From<Bytes> for Body {
    fn from(value: Bytes) -> Self {
        Body(BodyInner::Bytes(value))
    }
}

impl From<BytesMut> for Body {
    fn from(value: BytesMut) -> Self {
        Body(BodyInner::Bytes(value.into()))
    }
}

impl From<TryBoxStream<Bytes>> for Body {
    fn from(value: TryBoxStream<Bytes>) -> Self {
        Body(BodyInner::Stream(value))
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
