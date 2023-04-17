use crate::{error::Error, web::Request};
use http::header;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ContentTypeError {
    #[error("content type not found")]
    NoContentType,

    #[error("expected content type `{expected}` but was `{actual}`")]
    InvalidContentType {
        expected: mime::Mime,
        actual: mime::Mime,
    },

    #[error("failed to parse content type: {0}")]
    ParseError(Error),
}

pub(crate) fn validate_content_type(
    expected: mime::Mime,
    req: &Request,
) -> Result<(), ContentTypeError> {
    let header = req.headers().get(header::CONTENT_TYPE);
    let Some(header)  = header else {
        return Err(ContentTypeError::NoContentType);
    };

    let content_type = match header.to_str() {
        Ok(s) => s,
        Err(err) => {
            return Err(ContentTypeError::ParseError(err.into()));
        }
    };

    let mime = match mime::Mime::from_str(content_type) {
        Ok(m) => m,
        Err(err) => {
            return Err(ContentTypeError::ParseError(err.into()));
        }
    };

    if mime != expected {
        return Err(ContentTypeError::InvalidContentType {
            expected,
            actual: mime,
        });
    }

    Ok(())
}
