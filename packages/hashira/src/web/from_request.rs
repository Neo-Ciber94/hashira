use std::{
    convert::Infallible,
    future::{ready, Ready},
    task::Poll,
};

use crate::{
    app::{RequestContext, ResponseError},
    error::Error,
    routing::Params,
};
use futures::Future;
use http::{HeaderMap, Method, StatusCode, Uri, Version};

use super::{parse_body_to_bytes, ParseBodyOptions};

/// Provides a way for creating a type from a request.
pub trait FromRequest: Sized {
    /// The returned error on failure.
    type Error: Into<Error>;

    /// The future that resolves to the type.
    type Fut: Future<Output = Result<Self, Self::Error>>;

    /// Returns a future that resolves to the type or error.
    fn from_request(ctx: &RequestContext) -> Self::Fut;
}

impl FromRequest for RequestContext {
    type Error = Infallible;
    type Fut = Ready<Result<RequestContext, Infallible>>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        ready(Ok(ctx.clone()))
    }
}

impl FromRequest for () {
    type Error = Infallible;
    type Fut = Ready<Result<(), Self::Error>>;

    fn from_request(_ctx: &RequestContext) -> Self::Fut {
        ready(Ok(()))
    }
}

impl FromRequest for Method {
    type Error = Infallible;
    type Fut = Ready<Result<Method, Infallible>>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        ready(Ok(ctx.request().method().clone()))
    }
}

impl FromRequest for HeaderMap {
    type Error = Infallible;
    type Fut = Ready<Result<HeaderMap, Infallible>>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        ready(Ok(ctx.request().headers().clone()))
    }
}

impl FromRequest for Version {
    type Error = Infallible;
    type Fut = Ready<Result<Version, Infallible>>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        ready(Ok(ctx.request().version().clone()))
    }
}

impl FromRequest for Uri {
    type Error = Infallible;
    type Fut = Ready<Result<Uri, Infallible>>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        ready(Ok(ctx.request().uri().clone()))
    }
}

impl FromRequest for Params {
    type Error = Infallible;
    type Fut = Ready<Result<Params, Infallible>>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        ready(Ok(ctx.params().clone()))
    }
}

impl FromRequest for String {
    type Error = Error;
    type Fut = StringFromRequestFuture;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        StringFromRequestFuture(ctx.clone())
    }
}

#[doc(hidden)]
pub struct StringFromRequestFuture(RequestContext);
impl Future for StringFromRequestFuture {
    type Output = Result<String, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let opts = ParseBodyOptions { allow_empty: false };
        let bytes = match parse_body_to_bytes(self.0.request(), opts) {
            Ok(bytes) => bytes,
            Err(err) => return Poll::Ready(Err(err)),
        };

        match String::from_utf8(bytes.to_vec()) {
            Ok(s) => Poll::Ready(Ok(s)),
            Err(err) => Poll::Ready(Err(ResponseError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("failed to parse body: {err}"),
            )
            .into())),
        }
    }
}
