use bytes::Bytes;
use futures::{ready, Future, FutureExt};
use http::{header, HeaderValue, Method};
use pin_project_lite::pin_project;
use serde::{de::DeserializeOwned, Serialize};
use std::{marker::PhantomData, task::Poll};

use crate::{
    app::RequestContext,
    error::{BoxError, ServerError},
    responses,
    web::{Body, FromRequest, IntoResponse, Request, Response},
};

use super::utils::is_content_type;

/// Represents form data.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Form<T>(pub T);

impl<T> Form<T> {
    /// Returns the form value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

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
            Err(err) => ServerError::from_error(err).into_response(),
        }
    }
}

impl<T> FromRequest for Form<T>
where
    T: DeserializeOwned,
{
    type Error = BoxError;
    type Fut = FromRequestFormFuture<T>;

    fn from_request(ctx: &RequestContext, body: &mut Body) -> Self::Fut {
        FromRequestFormFuture {
            fut: Bytes::from_request(ctx, body),
            ctx: ctx.clone(),
            _marker: PhantomData,
        }
    }
}

pin_project! {
    #[doc(hidden)]
    pub struct FromRequestFormFuture<T> {
        #[pin]
        fut: <Bytes as FromRequest>::Fut,
        ctx: RequestContext,
        _marker: PhantomData<T>,
    }
}

impl<T> Future for FromRequestFormFuture<T>
where
    T: DeserializeOwned,
{
    type Output = Result<Form<T>, BoxError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let req = self.ctx.request();
        if let Err(err) = is_content_type(req, mime::APPLICATION_WWW_FORM_URLENCODED) {
            return Poll::Ready(Err(responses::unprocessable_entity(err)));
        }

        let request = self.ctx.request();
        let method = request.method();

        if method == Method::GET || method == Method::HEAD {
            return Poll::Ready(parse_form_from_uri(request));
        }

        let mut this = self.as_mut();
        let ret = ready!(this.fut.poll_unpin(cx));
        
        match ret {
            Ok(bytes) => match serde_urlencoded::from_bytes::<T>(&bytes) {
                Ok(x) => Poll::Ready(Ok(Form(x))),
                Err(err) => Poll::Ready(Err(responses::unprocessable_entity(format!(
                    "failed to deserialize form: {err}"
                )))),
            },
            Err(err) => Poll::Ready(Err(responses::unprocessable_entity(err))),
        }
    }
}

fn parse_form_from_uri<T: DeserializeOwned>(request: &Request<()>) -> Result<Form<T>, BoxError> {
    match request.uri().query() {
        Some(query) => match serde_urlencoded::from_str::<T>(query) {
            Ok(x) => Ok(Form(x)),
            Err(err) => Err(responses::unprocessable_entity(format!(
                "failed to deserialize uri query: {err}"
            ))),
        },
        None => Err(responses::unprocessable_entity("uri query not found")),
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
        web::{Body, Form, FromRequest, Request},
    };
    use http::{header, Method};
    use serde::Deserialize;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_form_from_request_body() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct MagicGirl {
            name: String,
            age: u32,
            dead: bool,
        }

        let req = Request::builder()
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .method(Method::POST)
            .body(())
            .unwrap();

        let ctx = create_request_context(req);
        let mut body =Body::from("name=Homura%20Akemi&age=14&dead=false");
        let form = Form::<MagicGirl>::from_request(&ctx, &mut body).await.unwrap();

        assert_eq!(
            form.0,
            MagicGirl {
                name: String::from("Homura Akemi"),
                age: 14,
                dead: false
            }
        );
    }

    #[tokio::test]
    async fn test_form_from_request_uri() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct MagicGirl {
            name: String,
            age: u32,
            dead: bool,
        }

        let req = Request::builder()
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .uri("/path/to/route?name=Homura%20Akemi&age=14&dead=false")
            .method(Method::GET)
            .body(())
            .unwrap();

        let ctx = create_request_context(req);
        let mut body = Body::default();
        let form = Form::<MagicGirl>::from_request(&ctx, &mut body).await.unwrap();

        assert_eq!(
            form.0,
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
