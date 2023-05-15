pub use either::*;
use futures::{ready, Future};
use pin_project_lite::pin_project;
use std::task::Poll;

use crate::{
    app::RequestContext,
    error::BoxError,
    web::{Body, Bytes, FromRequest, IntoResponse},
};

impl<L, R> IntoResponse for Either<L, R>
where
    L: IntoResponse,
    R: IntoResponse,
{
    fn into_response(self) -> crate::web::Response {
        match self {
            Left(left) => left.into_response(),
            Right(right) => right.into_response(),
        }
    }
}

impl<L: FromRequest, R: FromRequest> FromRequest for Either<L, R> {
    type Error = BoxError;
    type Fut = ExtractEitherFuture<L, R>;

    fn from_request(ctx: &RequestContext, body: &mut crate::web::Body) -> Self::Fut {
        ExtractEitherFuture {
            ctx: ctx.clone(),
            state: ExtractEitherState::Bytes {
                bytes: Bytes::from_request(ctx, body),
            },
        }
    }
}

pin_project! {

    pub struct ExtractEitherFuture<L: FromRequest, R: FromRequest> {
        ctx: RequestContext,

        #[pin]
        state: ExtractEitherState<L, R>
    }
}

pin_project! {
    #[project = EitherExtractProj]
    enum ExtractEitherState<L: FromRequest, R: FromRequest> {
        Bytes {
            #[pin]
            bytes: <Bytes as FromRequest>::Fut,
        },

        Left {
            #[pin]
            fut: <L as FromRequest>::Fut,
            buf: Bytes
        },

        Right {
            #[pin]
            fut: <R as FromRequest>::Fut,
        }
    }
}

impl<L, R> Future for ExtractEitherFuture<L, R>
where
    L: FromRequest,
    R: FromRequest,
{
    type Output = Result<Either<L, R>, BoxError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.project();
        let ready = loop {
            let state = this.state.as_mut().project();

            match state {
                EitherExtractProj::Bytes { bytes } => {
                    let ret = ready!(bytes.poll(cx));
                    match ret {
                        Ok(bytes) => {
                            let ctx = &this.ctx;
                            let mut body = Body::from(bytes.clone());
                            this.state.set(ExtractEitherState::Left {
                                fut: L::from_request(ctx, &mut body),
                                buf: bytes,
                            });
                        }
                        Err(err) => break Err(err),
                    }
                }
                EitherExtractProj::Left { fut, buf } => {
                    let ret = ready!(fut.poll(cx));
                    match ret {
                        Ok(result) => break Ok(Either::Left(result)),
                        Err(_) => {
                            let ctx = &this.ctx;
                            let bytes = std::mem::take(buf);
                            let mut body = Body::from(bytes);
                            this.state.set(ExtractEitherState::Right {
                                fut: R::from_request(ctx, &mut body),
                            });
                        }
                    }
                }
                EitherExtractProj::Right { fut } => {
                    let ret = ready!(fut.poll(cx));
                    match ret {
                        Ok(result) => break Ok(Either::Right(result)),
                        Err(err) => break Err(err.into()),
                    }
                }
            }
        };

        Poll::Ready(ready)
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
        web::{Body, Data, Either, FromRequest, Request},
    };
    use std::sync::Arc;

    pub struct State(usize);

    #[tokio::test]
    async fn either_from_left_test() {
        let mut app_data = AppData::default();
        app_data.insert(Data::new(State(12)));

        let ctx = create_request_context(app_data);
        let data = Either::<Data<State>, Data<String>>::from_request(&ctx, &mut Body::empty())
            .await
            .unwrap();

        assert_eq!(data.unwrap_left().0, 12);
    }

    #[tokio::test]
    async fn either_from_right_test() {
        let mut app_data = AppData::default();
        app_data.insert(Data::new(State(21)));

        let ctx = create_request_context(app_data);
        let data = Either::<Data<String>, Data<State>>::from_request(&ctx, &mut Body::empty())
            .await
            .unwrap();

        assert_eq!(data.unwrap_right().0, 21);
    }

    #[tokio::test]
    async fn not_match_test() {
        let app_data = AppData::default();

        let ctx = create_request_context(app_data);
        let data = Either::<Data<String>, Data<State>>::from_request(&ctx, &mut Body::empty())
            .await;

        assert!(data.is_err());
    }


    fn create_request_context(app_data: AppData) -> RequestContext {
        RequestContext::new(
            Arc::new(Request::new(())),
            app_data.into(),
            PageRouterWrapper::from(PageRouter::new()),
            Arc::new(ErrorRouter::new()),
            None,
            Params::default(),
        )
    }
}
