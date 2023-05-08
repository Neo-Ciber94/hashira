use futures::Future;
use pin_project_lite::pin_project;

use crate::{app::RequestContext, web::FromRequest};
use std::{convert::Infallible, marker::PhantomData, task::Poll};

impl<T, E> FromRequest for Result<T, E>
where
    T: FromRequest,
    T::Error: Into<E>,
{
    type Error = Infallible;
    type Fut = FromRequestResultFuture<T::Fut, E>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        FromRequestResultFuture {
            fut: T::from_request(ctx),
            _marker: PhantomData,
        }
    }
}

pin_project! {
    pub struct FromRequestResultFuture<Fut, E> {
        #[pin]
        fut: Fut,
        _marker: PhantomData<E>
    }
}

impl<Fut, T, E, E2> Future for FromRequestResultFuture<Fut, E>
where
    Fut: Future<Output = Result<T, E2>>,
    E2: Into<E>,
{
    type Output = Result<Result<T, E>, Infallible>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let res = futures::ready!(this.fut.poll(cx));
        Poll::Ready(Ok(res.map_err(Into::into)))
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
        web::{Body, FromRequest, Inject, Request}, error::Error,
    };

    #[tokio::test]
    async fn result_from_request_test() {
        let req = Request::builder().body(Body::empty()).unwrap();

        let ctx = create_request_context(req);
        let ret1 = Result::<Inject<String>, Error>::from_request(&ctx).await.unwrap();
        let ret2 = Result::<Method, Error>::from_request(&ctx).await.unwrap();

        assert!(ret1.is_err());
        assert!(ret2.is_ok());
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
