use crate::{types::BoxFuture, web::FromRequest};
use http::header;
use multer_derive::{Error, FromMultipart, MultipartForm};
use std::convert::Infallible;

/// Represents a multipart form.
pub struct Multipart<T>(T);

impl<T> Multipart<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromRequest for Multipart<T>
where
    T: FromMultipart,
{
    type Error = Error;
    type Fut = BoxFuture<Result<Multipart<T>, Self::Error>>;

    fn from_request(ctx: &crate::app::RequestContext) -> Self::Fut {
        let ctx = ctx.clone();
        Box::pin(async move {
            let Some(header_value) = ctx.request().headers().get(header::CONTENT_TYPE) else {
                return Err(Error::new("content type was not specified"));
            };

            log::debug!("Preparing multipart...");

            let content_type = header_value.to_str().map_err(Error::new)?;
            let boundary =
                multer_derive::multer::parse_boundary(content_type).map_err(Error::new)?;

            // TODO: We should be able to take the entire body
            log::debug!("Reading request multipart body");
            let body = ctx.request().body().try_as_bytes().map_err(Error::new)?;
            let bytes = body.to_vec();

            let multer = multer_derive::multer::Multipart::new(
                futures::stream::once(async move { Ok::<_, Infallible>(bytes) }),
                boundary,
            );

            log::debug!("Parsing multipart...");
            let multipart = MultipartForm::with_multipart(multer)
                .await
                .map_err(Error::new)?;

            log::debug!(
                "Converting multipart to: {}",
                std::any::type_name::<Multipart<T>>()
            );
            let value = T::from_multipart(&multipart, Default::default()).map_err(Error::new)?;
            Ok(Multipart(value))
        })
    }
}
