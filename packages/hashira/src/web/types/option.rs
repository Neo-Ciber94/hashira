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
