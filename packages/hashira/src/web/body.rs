use crate::{error::BoxError, responses, types::TryBoxStream};
use bytes::{BufMut, Bytes, BytesMut};
use futures::{
    future::{ready, Ready},
    StreamExt, TryStreamExt,
};
use std::{
    convert::Infallible,
    fmt::Debug,
    sync::{Arc, RwLock},
};
use thiserror::Error;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

use super::FromRequest;

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
pub struct Body(Arc<RwLock<Option<Payload>>>);

impl Body {
    /// Creates an empty body.
    pub fn empty() -> Self {
        let payload = Some(Payload::Bytes(Bytes::new()));
        let inner = Arc::new(RwLock::new(payload));
        Body(inner)
    }

    /// Creates a stream and returns a sender to add bytes to the body stream.
    pub fn stream() -> (UnboundedSender<Bytes>, Self) {
        let (tx, rx) = unbounded_channel::<Bytes>();

        let stream = UnboundedReceiverStream::new(rx)
            .map(Ok::<_, Infallible>)
            .map_err(|e| e.into());
        let body_stream = Box::pin(stream);
        let inner = Arc::new(RwLock::new(Some(Payload::Stream(body_stream))));
        (tx, Body(inner))
    }

    /// Returns the contents of the body leaving it empty.
    pub fn take(&self) -> Option<Payload> {
        self.0.write().unwrap().take()
    }

    /// Attempts to get the contents of the body.
    pub fn try_into_inner(self) -> Option<Payload> {
        self.take()
    }

    /// Returns contents of the body if available.
    ///
    /// # Panics
    /// If the content was taken.
    pub fn into_inner(self) -> Payload {
        self.take().expect("body contents was taken")
    }

    /// Returns the contents as bytes, or empty if was taken.
    pub async fn take_bytes(&self) -> Result<Bytes, BoxError> {
        match self.take() {
            Some(payload) => payload.into_bytes().await,
            None => Ok(Default::default()),
        }
    }

    /// Returns the contents as a stream, or empty if was taken.
    pub fn take_stream(&self) -> TryBoxStream<Bytes> {
        match self.take() {
            Some(payload) => payload.into_stream(),
            None => Box::pin(futures::stream::empty()),
        }
    }

    /// Returns a copy of the bytes of the body if can be read sequentially.
    pub fn try_to_bytes(&self) -> Result<Bytes, InvalidBodyError> {
        let lock = self.0.read().unwrap();
        if let Some(Payload::Bytes(bytes)) = lock.as_ref() {
            Ok(bytes.clone())
        } else {
            Err(InvalidBodyError::Stream)
        }
    }
}

impl Payload {
    /// Returns the contents as bytes.
    pub async fn into_bytes(self) -> Result<Bytes, BoxError> {
        match self {
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

    /// Returns the contents as a bytes stream.
    pub fn into_stream(self) -> TryBoxStream<Bytes> {
        match self {
            Payload::Bytes(bytes) => Box::pin(futures::stream::once(async move { Ok(bytes) })),
            Payload::Stream(stream) => stream,
        }
    }
}

impl FromRequest for Body {
    type Error = Infallible;
    type Fut = Ready<Result<Body, Infallible>>;

    fn from_request(_ctx: &crate::app::RequestContext, body: &mut Body) -> Self::Fut {
        let body = std::mem::take(body);
        ready(Ok(body))
    }
}

impl FromRequest for Payload {
    type Error = BoxError;
    type Fut = Ready<Result<Payload, BoxError>>;

    fn from_request(_ctx: &crate::app::RequestContext, body: &mut Body) -> Self::Fut {
        ready(
            body.take()
                .ok_or(responses::unprocessable_entity("body was taken")),
        )
    }
}

impl Default for Body {
    fn default() -> Self {
        Body::empty()
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Body")
    }
}

impl From<Bytes> for Body {
    fn from(value: Bytes) -> Self {
        let payload = Some(Payload::Bytes(value));
        let inner = Arc::new(RwLock::new(payload));
        Body(inner)
    }
}

impl From<BytesMut> for Body {
    fn from(value: BytesMut) -> Self {
        value.freeze().into()
    }
}

impl From<TryBoxStream<Bytes>> for Body {
    fn from(value: TryBoxStream<Bytes>) -> Self {
        let payload = Some(Payload::Stream(value));
        let inner = Arc::new(RwLock::new(payload));
        Body(inner)
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
