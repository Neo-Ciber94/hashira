use super::Request;
use crate::{app::ResponseError, error::Error};
use http::{header, StatusCode};
use std::str::FromStr;

mod json;
pub use json::*;

mod html;
pub use html::*;

mod redirect;
pub use redirect::*;

mod form;
pub use form::*;

pub(crate) fn check_content_type(expected: mime::Mime, req: &Request) -> Result<(), Error> {
    let header = req.headers().get(header::CONTENT_TYPE);
    let Some(header)  = header else {
        return Err(ResponseError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "content type not found",
        )
        .into());
    };

    let content_type = match header.to_str() {
        Ok(s) => s,
        Err(err) => {
            return Err(
                ResponseError::new(StatusCode::UNPROCESSABLE_ENTITY, err.to_string()).into(),
            );
        }
    };

    let mime = match mime::Mime::from_str(content_type) {
        Ok(m) => m,
        Err(err) => {
            return Err(ResponseError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("invalid content type: {err}"),
            )
            .into());
        }
    };

    if mime != expected {
        return Err(ResponseError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("invalid content type {mime}, expected {expected}"),
        )
        .into());
    }

    Ok(())
}
