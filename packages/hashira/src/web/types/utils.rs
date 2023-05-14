use crate::web::{Request, RequestExt};
use mime::Mime;
use thiserror::Error;

#[doc(hidden)]
#[derive(Debug, Error)]
#[error("expected content type `{expected}` but was `{actual}`")]
pub struct InvalidContentType {
    pub expected: mime::Mime,
    pub actual: mime::Mime,
}

#[derive(Debug, Error)]
pub(crate) enum ContentTypeError {
    #[error("content type not found")]
    NoContentType,

    #[error(transparent)]
    InvalidContentType(Box<InvalidContentType>),
}

pub(crate) fn is_content_type<B>(request: &Request<B>, expected: Mime) -> Result<(), ContentTypeError> {
    let content_type = request.content_type();
    let Some(content_type)  = content_type else {
        return Err(ContentTypeError::NoContentType);
    };

    if content_type.essence_str() != expected.essence_str() {
        let err = Box::new(InvalidContentType {
            actual: content_type,
            expected,
        });
        return Err(ContentTypeError::InvalidContentType(err));
    }

    Ok(())
}
