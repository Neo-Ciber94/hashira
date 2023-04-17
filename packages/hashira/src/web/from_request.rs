use crate::{
    app::{RequestContext, ResponseError},
    error::Error,
};
use bytes::Bytes;
use futures::Future;
use http::StatusCode;
use std::task::Poll;

/// Provides a way for creating a type from a request.
pub trait FromRequest: Sized {
    /// The returned error on failure.
    type Error: Into<Error>;

    /// The future that resolves to the type.
    type Fut: Future<Output = Result<Self, Self::Error>>;

    /// Returns a future that resolves to the type or error.
    fn from_request(ctx: RequestContext) -> Self::Fut;
}

impl FromRequest for Bytes {
    type Error = Error;
    type Fut = ExtractBytesFuture;

    fn from_request(ctx: RequestContext) -> Self::Fut {
        ExtractBytesFuture(ctx)
    }
}

#[doc(hidden)]
pub struct ExtractBytesFuture(RequestContext);
impl Future for ExtractBytesFuture {
    type Output = Result<Bytes, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let body = self.0.request().body();
        let bytes = match body.try_as_bytes() {
            Ok(b) => b,
            Err(err) => {
                return Poll::Ready(Err(ResponseError::unprocessable_entity(format!(
                    "invalid body: {err}"
                ))
                .into()))
            }
        };

        Poll::Ready(Ok(bytes.clone()))
    }
}

pub(crate) struct ParseBodyOptions {
    pub allow_empty: bool,
}

pub(crate) fn parse_body_to_bytes(
    req: &super::Request,
    opts: ParseBodyOptions,
) -> Result<Bytes, Error> {
    let body = req.body();
    match body.try_as_bytes().cloned() {
        Ok(bytes) => {
            if !opts.allow_empty && bytes.is_empty() {
                return Err(
                    ResponseError::new(StatusCode::UNPROCESSABLE_ENTITY, "body is empty").into(),
                );
            }

            Ok(bytes)
        }
        Err(err) => {
            return Err(ResponseError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("invalid body: {err}"),
            )
            .into())
        }
    }
}
