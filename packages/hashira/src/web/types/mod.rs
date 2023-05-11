pub mod utils;

mod query;
use std::fmt::Display;

pub use query::*;

mod json;
pub use json::*;

mod html;
pub use html::*;

mod redirect;
pub use redirect::*;

mod form;
pub use form::*;

mod bytes_;
pub use bytes_::*;

mod option;
pub use option::*;

mod result;
pub use result::*;

mod data;
pub use data::*;

mod inject;
pub use inject::*;

#[inline]
pub(crate) fn unprocessable_entity_error(err: impl Display) -> crate::error::Error {
    use crate::{error::ServerError, web::status::StatusCode};

    ServerError::new(StatusCode::UNPROCESSABLE_ENTITY, err.to_string()).into()
}
