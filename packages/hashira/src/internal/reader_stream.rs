// Adapted from: https://docs.rs/tokio-util/latest/src/tokio_util/io/reader_stream.rs.html#10-55

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::{Bytes, BytesMut};
use futures::{AsyncRead, Stream, TryStreamExt};
use pin_project_lite::pin_project;

use crate::types::TryBoxStream;

const DEFAULT_CAPACITY: usize = 4096;

pin_project! {
    #[derive(Debug)]
    pub struct ReaderStream<R> {
        #[pin]
        reader: Option<R>,
        buf: BytesMut,
        capacity: usize,
    }
}

impl<R: AsyncRead> ReaderStream<R> {
    pub fn new(reader: R) -> Self {
        ReaderStream {
            reader: Some(reader),
            buf: BytesMut::new(),
            capacity: DEFAULT_CAPACITY,
        }
    }

    pub fn with_capacity(reader: R, capacity: usize) -> Self {
        ReaderStream {
            reader: Some(reader),
            buf: BytesMut::with_capacity(capacity),
            capacity,
        }
    }
}

impl<R: AsyncRead> Stream for ReaderStream<R> {
    type Item = std::io::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().project();

        let reader = match this.reader.as_pin_mut() {
            Some(r) => r,
            None => return Poll::Ready(None),
        };

        if this.buf.capacity() == 0 {
            this.buf.reserve(*this.capacity);
        }

        let buf = this.buf;
        match reader.poll_read(cx, buf) {
            Poll::Ready(Ok(0)) => {
                self.project().reader.set(None);
                Poll::Ready(None)
            }
            Poll::Ready(Ok(_)) => {
                let chunk = buf.split();
                Poll::Ready(Some(Ok(chunk.freeze())))
            }
            Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Converts a `futures-io` `AsyncRead` to a `Stream<Item = Result<Bytes, BoxError>>`.
pub fn to_stream<R>(reader: R) -> TryBoxStream<Bytes>
where
    R: AsyncRead + Send + Sync + 'static,
{
    Box::pin(ReaderStream::new(reader).map_err(Into::into))
}
