use std::task::Poll;

use bytes::Bytes;
use futures::{ready, Future, FutureExt};
use pin_project_lite::pin_project;

use crate::{app::RequestContext, error::BoxError, responses, web::{FromRequest, Body}};

impl FromRequest for String {
    type Error = BoxError;
    type Fut = StringFromRequestFuture;

    fn from_request(ctx: &RequestContext, body: &mut Body) -> Self::Fut {
        StringFromRequestFuture {
            fut: Bytes::from_request(ctx, body),
        }
    }
}

pin_project! {
    #[doc(hidden)]
    pub struct StringFromRequestFuture {
        #[pin]
        fut: <Bytes as FromRequest>::Fut
    }
}

impl Future for StringFromRequestFuture {
    type Output = Result<String, BoxError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.as_mut();
        let ret = ready!(this.fut.poll_unpin(cx));

        match ret {
            Ok(bytes) => match String::from_utf8(bytes.to_vec()) {
                Ok(s) => Poll::Ready(Ok(s)),
                Err(err) => {
                    let err = responses::unprocessable_entity(err);
                    Poll::Ready(Err(err))
                }
            },
            Err(err) => Poll::Ready(Err(responses::unprocessable_entity(err))),
        }
    }
}
