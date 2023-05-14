use futures::Future;
use pin_project_lite::pin_project;

use crate::{
    app::RequestContext,
    web::{Body, FromRequest},
};
use std::{convert::Infallible, marker::PhantomData, task::Poll};

impl<T, E> FromRequest for Result<T, E>
where
    T: FromRequest,
    T::Error: Into<E>,
{
    type Error = Infallible;
    type Fut = FromRequestResultFuture<T::Fut, E>;

    fn from_request(ctx: &RequestContext, body: &mut Body) -> Self::Fut {
        FromRequestResultFuture {
            fut: T::from_request(ctx, body),
            _marker: PhantomData,
        }
    }
}

pin_project! {
    #[doc(hidden)]
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
    use crate::{
        app::{
            router::{PageRouter, PageRouterWrapper},
            AppData, RequestContext,
        },
        error::BoxError,
        routing::{ErrorRouter, Params},
        web::{Body, FromRequest, Inject, Request},
    };
    use http::Method;
    use std::sync::Arc;

    #[tokio::test]
    async fn result_from_request_test() {
        let req = Request::new(());

        let ctx = create_request_context(req);
        let ret1 = Result::<Inject<String>, BoxError>::from_request(&ctx, &mut Body::empty())
            .await
            .unwrap();
        let ret2 = Result::<Method, BoxError>::from_request(&ctx, &mut Body::empty())
            .await
            .unwrap();

        assert!(ret1.is_err());
        assert!(ret2.is_ok());
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
