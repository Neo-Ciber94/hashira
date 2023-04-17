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

    fn from_request(ctx: RequestContext) -> Self::Fut {
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
