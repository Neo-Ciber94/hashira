use futures::Future;
use http::{header, HeaderValue};
use serde::{de::DeserializeOwned, Serialize};
use std::{marker::PhantomData, task::Poll};

use crate::{
    app::{RequestContext, ResponseError},
    error::Error,
    web::{parse_body_to_bytes, Body, FromRequest, IntoResponse, ParseBodyOptions, Response},
};

use super::utils::validate_content_type;

/// Represents form data.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Form<T>(pub T);

impl<T> IntoResponse for Form<T>
where
    T: Serialize,
{
    fn into_response(self) -> crate::web::Response {
        match serde_urlencoded::to_string(self.0) {
            Ok(s) => {
                let content_type = mime::WWW_FORM_URLENCODED.to_string();
                let header_value = HeaderValue::from_str(&content_type).unwrap();
                Response::builder()
                    .header(header::CONTENT_TYPE, header_value)
                    .body(Body::from(s))
                    .unwrap()
            }
            Err(err) => ResponseError::from_error(err).into_response(),
        }
    }
}

impl<T> FromRequest for Form<T>
where
    T: DeserializeOwned,
{
    type Error = Error;
    type Fut = ExtractFormFuture<T>;

    fn from_request(ctx: crate::app::RequestContext) -> Self::Fut {
        ExtractFormFuture {
            ctx,
            _marker: PhantomData,
        }
    }
}

#[doc(hidden)]
pub struct ExtractFormFuture<T> {
    ctx: RequestContext,
    _marker: PhantomData<T>,
}

impl<T> Future for ExtractFormFuture<T>
where
    T: DeserializeOwned,
{
    type Output = Result<Form<T>, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Err(err) = validate_content_type(
            mime::APPLICATION_WWW_FORM_URLENCODED.into(),
            self.ctx.request(),
        ) {
            return Poll::Ready(Err(ResponseError::unprocessable_entity(err).into()));
        }

        let opts = ParseBodyOptions { allow_empty: false };
        let bytes = match parse_body_to_bytes(self.ctx.request(), opts) {
            Ok(bytes) => bytes,
            Err(err) => return Poll::Ready(Err(err)),
        };

        match serde_urlencoded::from_bytes::<T>(&bytes) {
            Ok(x) => Poll::Ready(Ok(Form(x))),
            Err(err) => Poll::Ready(Err(ResponseError::unprocessable_entity(format!(
                "failed to deserialize form: {err}"
            ))
            .into())),
        }
    }
}
