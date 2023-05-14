use crate::{
    app::RequestContext,
    error::{BoxError, ServerError},
    responses,
    web::{Body, FromRequest, IntoResponse, Response},
};
use bytes::Bytes;
use futures::{ready, Future, FutureExt};
use http::header;
use pin_project_lite::pin_project;
use serde::{de::DeserializeOwned, Serialize};
use std::{marker::PhantomData, task::Poll};

use super::utils::is_content_type;

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
                return ServerError::from_error(err).into_response();
            }
        };

        let body = Body::from(json);
        let mut res = Response::new(body);
        res.headers_mut().append(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        res
    }
}

impl<T> FromRequest for Json<T>
where
    T: DeserializeOwned,
{
    type Error = BoxError;
    type Fut = FromRequestJsonFuture<T>;

    fn from_request(ctx: &RequestContext, body: &mut Body) -> Self::Fut {
        FromRequestJsonFuture {
            fut: Bytes::from_request(ctx, body),
            ctx: ctx.clone(),
            _marker: PhantomData,
        }
    }
}

pin_project! {
    #[doc(hidden)]
    pub struct FromRequestJsonFuture<T> {
        #[pin]
        fut: <Bytes as FromRequest>::Fut,
        ctx: RequestContext,
        _marker: PhantomData<T>,
    }
}

impl<T> Future for FromRequestJsonFuture<T>
where
    T: DeserializeOwned,
{
    type Output = Result<Json<T>, BoxError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let req = self.ctx.request();
        if let Err(err) = is_content_type(req, mime::APPLICATION_JSON) {
            return Poll::Ready(Err(responses::unprocessable_entity(err)));
        }

        let mut this = self.as_mut();
        let ret = ready!(this.fut.poll_unpin(cx));

        match ret {
            Ok(bytes) => match serde_json::from_slice::<T>(&bytes) {
                Ok(x) => Poll::Ready(Ok(Json(x))),
                Err(err) => Poll::Ready(Err(responses::unprocessable_entity(format!(
                    "failed to deserialize json: {err}"
                )))),
            },
            Err(err) => Poll::Ready(Err(responses::unprocessable_entity(err))),
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
        routing::{ErrorRouter, Params},
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
            .body(())
            .unwrap();

        let ctx = create_request_context(req);
        let mut body = Body::from(
            r#"{
            "name": "Homura Akemi",
            "age": 14,
            "dead": false 
        }"#);
        let json = Json::<MagicGirl>::from_request(&ctx, &mut body)
            .await
            .unwrap();

        assert_eq!(
            json.into_inner(),
            MagicGirl {
                name: String::from("Homura Akemi"),
                age: 14,
                dead: false
            }
        );
    }

    fn create_request_context(req: Request<()>) -> RequestContext {
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
