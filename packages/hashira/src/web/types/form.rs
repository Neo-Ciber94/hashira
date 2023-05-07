use futures::Future;
use http::{header, HeaderValue, Method};
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
            Err(err) => ResponseError::with_error(err).into_response(),
        }
    }
}

impl<T> FromRequest for Form<T>
where
    T: DeserializeOwned,
{
    type Error = Error;
    type Fut = FromRequestFormFuture<T>;

    fn from_request(ctx: &crate::app::RequestContext) -> Self::Fut {
        FromRequestFormFuture {
            ctx: ctx.clone(),
            _marker: PhantomData,
        }
    }
}

#[doc(hidden)]
pub struct FromRequestFormFuture<T> {
    ctx: RequestContext,
    _marker: PhantomData<T>,
}

impl<T> Future for FromRequestFormFuture<T>
where
    T: DeserializeOwned,
{
    type Output = Result<Form<T>, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Err(err) =
            validate_content_type(mime::APPLICATION_WWW_FORM_URLENCODED, self.ctx.request())
        {
            return Poll::Ready(Err(ResponseError::unprocessable_entity(err).into()));
        }

        let request = self.ctx.request();
        let method = request.method();
        if method == Method::GET || method == Method::HEAD {
            return match request.uri().query() {
                Some(query) => match serde_urlencoded::from_str::<T>(query) {
                    Ok(x) => Poll::Ready(Ok(Form(x))),
                    Err(err) => Poll::Ready(Err(ResponseError::unprocessable_entity(format!(
                        "failed to deserialize uri query: {err}"
                    ))
                    .into())),
                },
                None => Poll::Ready(Err(ResponseError::unprocessable_entity(
                    "uri query not found",
                )
                .into())),
            };
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
    use std::{sync::Arc};

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
            .body(Body::from("name=Homura%20Akemi&age=14&dead=false"))
            .unwrap();

        let ctx = create_request_context(req);
        let form = Form::<MagicGirl>::from_request(&ctx).await.unwrap();

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
            .body(Body::empty())
            .unwrap();

        let ctx = create_request_context(req);
        let form = Form::<MagicGirl>::from_request(&ctx).await.unwrap();

        assert_eq!(
            form.0,
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
