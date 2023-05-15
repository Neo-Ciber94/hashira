use crate::{
    app::RequestContext,
    error::BoxError,
    types::BoxFuture,
    web::{Body, FromRequest, IntoResponse, Response},
};
use bytes::{Bytes, BytesMut};
use http::header;

impl FromRequest for Bytes {
    type Error = BoxError;
    type Fut = BoxFuture<Result<Bytes, Self::Error>>;

    fn from_request(_ctx: &RequestContext, body: &mut Body) -> Self::Fut {
        let body = std::mem::take(body);
        Box::pin(async move {
            let bytes = body.into_bytes().await?;
            Ok(bytes)
        })
    }
}

impl IntoResponse for Bytes {
    fn into_response(self) -> crate::web::Response {
        Response::builder()
            .header(
                header::CONTENT_TYPE,
                mime::APPLICATION_OCTET_STREAM.essence_str(),
            )
            .body(Body::from(self))
            .unwrap()
    }
}

impl IntoResponse for BytesMut {
    fn into_response(self) -> crate::web::Response {
        self.freeze().into_response()
    }
}
