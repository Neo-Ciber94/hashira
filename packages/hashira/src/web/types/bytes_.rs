use crate::{
    app::RequestContext,
    error::BoxError,
    types::BoxFuture,
    web::{Body, FromRequest},
};
use bytes::Bytes;

impl FromRequest for Bytes {
    type Error = BoxError;
    type Fut = BoxFuture<Result<Bytes, Self::Error>>;

    fn from_request(_ctx: &RequestContext, body: &mut Body) -> Self::Fut {
        let body = std::mem::take(body);
        Box::pin(async move {
            let bytes = body.take_bytes().await?;
            Ok(bytes)
        })
    }
}
