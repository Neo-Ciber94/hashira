#![doc(hidden)]
#![allow(non_snake_case)]

// Adapted from: https://docs.rs/actix-web/latest/src/actix_web/extract.rs.html#413

use crate::{
    app::RequestContext,
    error::BoxError,
    web::{Body, FromRequest},
};
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
            type Error = BoxError;
            type Fut = $fut<$($T),+>;

            fn from_request(ctx: &RequestContext, body: &mut Body) -> Self::Fut {
                $fut {
                    $(
                        $T: ExtractFuture::Future {
                            fut: $T::from_request(ctx, body)
                        },
                    )+
                }
            }
        }

        pin_project! {
            #[doc(hidden)]
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

tuple_from_req! { TupleFromRequest1;  T1 }
tuple_from_req! { TupleFromRequest2;  T1, T2 }
tuple_from_req! { TupleFromRequest3;  T1, T2, T3 }
tuple_from_req! { TupleFromRequest4;  T1, T2, T3, T4 }
tuple_from_req! { TupleFromRequest5;  T1, T2, T3, T4, T5 }
tuple_from_req! { TupleFromRequest6;  T1, T2, T3, T4, T5, T6 }
tuple_from_req! { TupleFromRequest7;  T1, T2, T3, T4, T5, T6, T7 }
tuple_from_req! { TupleFromRequest8;  T1, T2, T3, T4, T5, T6, T7, T8 }
tuple_from_req! { TupleFromRequest9;  T1, T2, T3, T4, T5, T6, T7, T8, T9 }
tuple_from_req! { TupleFromRequest10; T1, T2, T3, T4, T5, T6, T7, T8, T9, T10 }
tuple_from_req! { TupleFromRequest11; T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11 }
tuple_from_req! { TupleFromRequest12; T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12 }
