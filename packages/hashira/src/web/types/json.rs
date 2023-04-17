use futures::Future;
use http::{header, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::{marker::PhantomData, task::Poll};

use crate::{
    app::{RequestContext, ResponseError},
    error::Error,
    web::{parse_body_to_bytes, Body, FromRequest, IntoResponse, ParseBodyOptions, Response},
};

use super::check_content_type;

/// Represents a JSON.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Json<T>(pub T);
impl<T> Json<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        let json = match serde_json::to_string(&self.0) {
            Ok(s) => s,
            Err(err) => {
                return ResponseError::from_error(err).into_response();
            }
        };

        let body = Body::from(json);
        let mut res = Response::new(body);
        res.headers_mut().append(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json; charset=utf-8"),
        );
        res
    }
}

impl<T> FromRequest for Json<T>
where
    T: DeserializeOwned,
{
    type Error = Error;
    type Fut = ExtractJsonFuture<T>;

    fn from_request(ctx: RequestContext) -> Self::Fut {
        ExtractJsonFuture {
            ctx,
            _marker: PhantomData,
        }
    }
}

#[doc(hidden)]
pub struct ExtractJsonFuture<T> {
    ctx: RequestContext,
    _marker: PhantomData<T>,
}

impl<T> Future for ExtractJsonFuture<T>
where
    T: DeserializeOwned,
{
    type Output = Result<Json<T>, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Err(err) = check_content_type(mime::APPLICATION_JSON, self.ctx.request()) {
            return Poll::Ready(Err(err));
        }

        let opts = ParseBodyOptions { allow_empty: false };
        let bytes = match parse_body_to_bytes(self.ctx.request(), opts) {
            Ok(bytes) => bytes,
            Err(err) => return Poll::Ready(Err(err)),
        };

        match serde_json::from_slice::<T>(&bytes) {
            Ok(x) => Poll::Ready(Ok(Json(x))),
            Err(err) => Poll::Ready(Err(ResponseError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("failed to deserialize json: {err}"),
            )
            .into())),
        }
    }
}