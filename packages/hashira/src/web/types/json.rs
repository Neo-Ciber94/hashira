use futures::Future;
use http::{header, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::{marker::PhantomData, task::Poll};

use crate::{
    app::{RequestContext, ResponseError},
    error::Error,
    web::{parse_body_to_bytes, Body, FromRequest, IntoResponse, ParseBodyOptions, Response},
};

use super::utils::validate_content_type;

/// Represents a JSON.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Json<T>(pub T);
impl<T> Json<T> {
    /// Returns the json value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        let json = match serde_json::to_string(&self.0) {
            Ok(s) => s,
            Err(err) => {
                return ResponseError::with_error(err).into_response();
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
    type Fut = FromRequestJsonFuture<T>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        FromRequestJsonFuture {
            ctx: ctx.clone(),
            _marker: PhantomData,
        }
    }
}

#[doc(hidden)]
pub struct FromRequestJsonFuture<T> {
    ctx: RequestContext,
    _marker: PhantomData<T>,
}

impl<T> Future for FromRequestJsonFuture<T>
where
    T: DeserializeOwned,
{
    type Output = Result<Json<T>, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Err(err) = validate_content_type(mime::APPLICATION_JSON, self.ctx.request()) {
            return Poll::Ready(Err(ResponseError::unprocessable_entity(err).into()));
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

#[cfg(test)]
mod tests {
    use crate::{
        app::{
            router::{PageRouter, PageRouterWrapper},
            AppData, RequestContext,
        },
        routing::{Params, ErrorRouter},
        web::{Body, FromRequest, Json, Request},
    };
    use http::header;
    use serde::Deserialize;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_json_from_request() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct MagicGirl {
            name: String,
            age: u32,
            dead: bool,
        }

        let req = Request::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{
                "name": "Homura Akemi",
                "age": 14,
                "dead": false 
            }"#,
            ))
            .unwrap();

        let ctx = create_request_context(req);
        let json = Json::<MagicGirl>::from_request(&ctx).await.unwrap();

        assert_eq!(
            json.into_inner(),
            MagicGirl {
                name: String::from("Homura Akemi"),
                age: 14,
                dead: false
            }
        );
    }

    fn create_request_context(req: Request) -> RequestContext {
        RequestContext::new(
            Arc::new(req),
            Arc::new(AppData::default()),
            PageRouterWrapper::from(PageRouter::new()),
            Arc::new(ErrorRouter::new()),
            None,
            Params::default(),
        )
    }
}
