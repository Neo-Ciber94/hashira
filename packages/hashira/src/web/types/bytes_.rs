use crate::{app::RequestContext, error::Error, types::BoxFuture, web::FromRequest};
use bytes::Bytes;

impl FromRequest for Bytes {
    type Error = Error;
    type Fut = BoxFuture<Result<Bytes, Self::Error>>;

    fn from_request(ctx: &RequestContext) -> Self::Fut {
        let ctx = ctx.clone();
        Box::pin(async move {
            let bytes = ctx.request().body().take_bytes().await?;
            Ok(bytes)
        })
    }
}
