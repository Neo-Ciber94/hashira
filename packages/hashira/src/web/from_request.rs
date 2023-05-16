use std::{
    convert::Infallible,
    future::{ready, Ready},
};

use crate::{app::RequestContext, error::BoxError, routing::Params};
use futures::Future;
use http::{HeaderMap, Method, Uri, Version};

use super::Body;

/// Provides a way for creating a type from a request.
pub trait FromRequest: Sized {
    /// The returned error on failure.
    type Error: Into<BoxError>;

    /// The future that resolves to the type.
    type Fut: Future<Output = Result<Self, Self::Error>>;

    /// Returns a future that resolves to the type or error.
    fn from_request(ctx: &RequestContext, body: &mut Body) -> Self::Fut;
}

impl FromRequest for RequestContext {
    type Error = Infallible;
    type Fut = Ready<Result<RequestContext, Infallible>>;

    fn from_request(ctx: &RequestContext, _body: &mut Body) -> Self::Fut {
        ready(Ok(ctx.clone()))
    }
}

impl FromRequest for () {
    type Error = Infallible;
    type Fut = Ready<Result<(), Self::Error>>;

    fn from_request(_ctx: &RequestContext, _body: &mut Body) -> Self::Fut {
        ready(Ok(()))
    }
}

impl FromRequest for Method {
    type Error = Infallible;
    type Fut = Ready<Result<Method, Infallible>>;

    fn from_request(ctx: &RequestContext, _body: &mut Body) -> Self::Fut {
        ready(Ok(ctx.request().method().clone()))
    }
}

impl FromRequest for HeaderMap {
    type Error = Infallible;
    type Fut = Ready<Result<HeaderMap, Infallible>>;

    fn from_request(ctx: &RequestContext, _body: &mut Body) -> Self::Fut {
        ready(Ok(ctx.request().headers().clone()))
    }
}

impl FromRequest for Version {
    type Error = Infallible;
    type Fut = Ready<Result<Version, Infallible>>;

    fn from_request(ctx: &RequestContext, _body: &mut Body) -> Self::Fut {
        ready(Ok(ctx.request().version()))
    }
}

impl FromRequest for Uri {
    type Error = Infallible;
    type Fut = Ready<Result<Uri, Infallible>>;

    fn from_request(ctx: &RequestContext, _body: &mut Body) -> Self::Fut {
        ready(Ok(ctx.request().uri().clone()))
    }
}

impl FromRequest for Params {
    type Error = Infallible;
    type Fut = Ready<Result<Params, Infallible>>;

    fn from_request(ctx: &RequestContext, _body: &mut Body) -> Self::Fut {
        ready(Ok(ctx.params().clone()))
    }
}
