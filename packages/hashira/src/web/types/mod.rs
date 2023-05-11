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

mod multipart;
pub use multipart::*;

#[macro_export(local_inner_macros)]
macro_rules! ready_or_else {
    ($expr:expr, $ident:ident => $error:expr) => {
        match $expr {
            Ok(x) => x,
            Err($ident) => {
                return Poll::Ready(Err($error));
            }
        }
    };
}


#[inline]
pub(crate) fn unprocessable_entity_error(err: impl Display) -> crate::error::Error {
    use crate::{error::ServerError, web::status::StatusCode};

    ServerError::new(StatusCode::UNPROCESSABLE_ENTITY, err.to_string()).into()
}
