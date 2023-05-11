use std::{
    convert::Infallible,
    future::{ready, Ready},
    task::Poll,
};

use crate::{
    app::RequestContext,
    error::{Error, ServerError},
    routing::Params,
};
use futures::Future;
use http::{HeaderMap, Method, StatusCode, Uri, Version};

use super::{parse_body_to_bytes, ParseBodyOptions};

/// Provides a way for creating a type from a request.
pub trait FromRequest: Sized {
    /// The returned error on failure.
    type Error: Into<Error>;

    /// The future that resolves to the type.
    type Fut: Future<Output = Result<Self, Self::Error>>;

    /// Returns a future that resolves to the type or error.
    fn from_request(ctx: &RequestContext) -> Self::Fut;
}

impl FromRequest for RequestContext {
    type Error = Infallible;
    type Fut = Ready<Result<RequestContext, Infallible>>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        ready(Ok(ctx.clone()))
    }
}

impl FromRequest for () {
    type Error = Infallible;
    type Fut = Ready<Result<(), Self::Error>>;

    fn from_request(_ctx: &RequestContext) -> Self::Fut {
        ready(Ok(()))
    }
}

impl FromRequest for Method {
    type Error = Infallible;
    type Fut = Ready<Result<Method, Infallible>>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        ready(Ok(ctx.request().method().clone()))
    }
}

impl FromRequest for HeaderMap {
    type Error = Infallible;
    type Fut = Ready<Result<HeaderMap, Infallible>>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        ready(Ok(ctx.request().headers().clone()))
    }
}

impl FromRequest for Version {
    type Error = Infallible;
    type Fut = Ready<Result<Version, Infallible>>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        ready(Ok(ctx.request().version()))
    }
}

impl FromRequest for Uri {
    type Error = Infallible;
    type Fut = Ready<Result<Uri, Infallible>>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        ready(Ok(ctx.request().uri().clone()))
    }
}

impl FromRequest for Params {
    type Error = Infallible;
    type Fut = Ready<Result<Params, Infallible>>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        ready(Ok(ctx.params().clone()))
    }
}

impl FromRequest for String {
    type Error = Error;
    type Fut = StringFromRequestFuture;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        StringFromRequestFuture(ctx.clone())
    }
}

#[doc(hidden)]
pub struct StringFromRequestFuture(RequestContext);
impl Future for StringFromRequestFuture {
    type Output = Result<String, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let opts = ParseBodyOptions { allow_empty: false };
        let bytes = match parse_body_to_bytes(self.0.request(), opts) {
            Ok(bytes) => bytes,
            Err(err) => return Poll::Ready(Err(err)),
        };

        match String::from_utf8(bytes.to_vec()) {
            Ok(s) => Poll::Ready(Ok(s)),
            Err(err) => {
                let err = ServerError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("failed to parse body: {err}"),
                )
                .unwrap()
                .into();
                Poll::Ready(Err(err))
            }
        }
    }
}

// Adapted from: https://docs.rs/actix-web/latest/src/actix_web/extract.rs.html#413

#[doc(hidden)]
#[allow(non_snake_case)]
mod tuple_from_req {
    use super::{FromRequest, RequestContext};
    use crate::error::Error;
    use pin_project_lite::pin_project;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    macro_rules! tuple_from_req {
        ($fut: ident; $($T: ident),*) => {
            /// FromRequest implementation for tuple
            #[allow(unused_parens)]
            impl<$($T: FromRequest + 'static),+> FromRequest for ($($T,)+)
            {
                type Error = Error;
                type Fut = $fut<$($T),+>;

                fn from_request(ctx: &RequestContext) -> Self::Fut {
                    $fut {
                        $(
                            $T: ExtractFuture::Future {
                                fut: $T::from_request(ctx)
                            },
                        )+
                    }
                }
            }

            pin_project! {
                pub struct $fut<$($T: FromRequest),+> {
                    $(
                        #[pin]
                        $T: ExtractFuture<$T::Fut, $T>,
                    )+
                }
            }

            impl<$($T: FromRequest),+> Future for $fut<$($T),+>
            {
                type Output = crate::Result<($($T,)+),>;

                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                    let mut this = self.project();

                    let mut ready = true;
                    $(
                        match this.$T.as_mut().project() {
                            ExtractProj::Future { fut } => match fut.poll(cx) {
                                Poll::Ready(Ok(output)) => {
                                    let _ = this.$T.as_mut().project_replace(ExtractFuture::Done { output });
                                },
                                Poll::Ready(Err(e)) => return Poll::Ready(Err(e.into())),
                                Poll::Pending => ready = false,
                            },
                            ExtractProj::Done { .. } => {},
                            ExtractProj::Empty => unreachable!("FromRequest polled after finished"),
                        }
                    )+

                    if ready {
                        Poll::Ready(Ok(
                            ($(
                                match this.$T.project_replace(ExtractFuture::Empty) {
                                    ExtractReplaceProj::Done { output } => output,
                                    _ => unreachable!("FromRequest polled after finished"),
                                },
                            )+)
                        ))
                    } else {
                        Poll::Pending
                    }
                }
            }
        };
    }

    pin_project! {
        #[project = ExtractProj]
        #[project_replace = ExtractReplaceProj]
        enum ExtractFuture<Fut, Res> {
            Future {
                #[pin]
                fut: Fut
            },
            Done {
                output: Res,
            },
            Empty
        }
    }

    tuple_from_req! { TupleFromRequest1; A }
    tuple_from_req! { TupleFromRequest2; A, B }
    tuple_from_req! { TupleFromRequest3; A, B, C }
    tuple_from_req! { TupleFromRequest4; A, B, C, D }
    tuple_from_req! { TupleFromRequest5; A, B, C, D, E }
    tuple_from_req! { TupleFromRequest6; A, B, C, D, E, F }
    tuple_from_req! { TupleFromRequest7; A, B, C, D, E, F, G }
    tuple_from_req! { TupleFromRequest8; A, B, C, D, E, F, G, H }
    tuple_from_req! { TupleFromRequest9; A, B, C, D, E, F, G, H, I }
    tuple_from_req! { TupleFromRequest10; A, B, C, D, E, F, G, H, I, J }
    tuple_from_req! { TupleFromRequest11; A, B, C, D, E, F, G, H, I, J, K }
    tuple_from_req! { TupleFromRequest12; A, B, C, D, E, F, G, H, I, J, K, L }
}
