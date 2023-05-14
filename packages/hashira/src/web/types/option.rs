use futures::Future;
use pin_project_lite::pin_project;

use crate::{app::RequestContext, error::Error, web::FromRequest};
use std::{convert::Infallible, task::Poll};

impl<T> FromRequest for Option<T>
where
    T: FromRequest,
{
    type Error = Infallible;
    type Fut = FromRequestOptionFuture<T::Fut>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        FromRequestOptionFuture {
            fut: T::from_request(ctx),
        }
    }
}

pin_project! {
    #[doc(hidden)]
    pub struct FromRequestOptionFuture<Fut> {
        #[pin]
        fut: Fut,
    }
}

impl<Fut, T, E> Future for FromRequestOptionFuture<Fut>
where
    Fut: Future<Output = Result<T, E>>,
    E: Into<Error>,
{
    type Output = Result<Option<T>, Infallible>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let res = futures::ready!(this.fut.poll(cx));
        match res {
            Ok(x) => Poll::Ready(Ok(Some(x))),
            Err(_) => Poll::Ready(Ok(None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use http::Method;
    use crate::{
        app::{
            router::{PageRouter, PageRouterWrapper},
            AppData, RequestContext,
        },
        routing::{ErrorRouter, Params},
        web::{Body, FromRequest, Inject, Request},
    };

    #[tokio::test]
    async fn option_from_request_test() {
        let req = Request::builder().body(Body::empty()).unwrap();

        let ctx = create_request_context(req);
        let ret1 = Option::<Inject<String>>::from_request(&ctx).await.unwrap();
        let ret2 = Option::<Method>::from_request(&ctx).await.unwrap();

        assert!(ret1.is_none());
        assert!(ret2.is_some());
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
