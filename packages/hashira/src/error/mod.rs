mod js_error;
pub use js_error::*;

use crate::web::{FromRequest, IntoResponse, Json, Response};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, future::Ready};

/// A boxed error.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A error that occurred while processing a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    #[serde(with = "crate::web::serde::status_code")]
    status: StatusCode,
    message: Option<String>,
}

impl ResponseError {
    /// Constructs a new `ResponseError`.
    pub fn new(status: StatusCode, message: impl Display) -> Self {
        ResponseError {
            status,
            message: Some(message.to_string()),
        }
    }

    /// Constructs a new `ResponseError` from an error and the given `StatusCode`.
    pub fn with_error_and_status<E: Into<Error>>(status: StatusCode, error: E) -> Self {
        let err = error.into();
        match err.downcast::<ResponseError>() {
            Ok(err) => *err,
            Err(err) => ResponseError {
                status,
                message: Some(err.to_string()),
            },
        }
    }

    /// Constructs a new `ResponseError` from an error.
    pub fn with_error<E: Into<Error>>(error: E) -> Self {
        Self::with_error_and_status(StatusCode::INTERNAL_SERVER_ERROR, error)
    }

    /// Constructs an error from the given status code.
    pub fn from_status(status: StatusCode) -> Self {
        ResponseError {
            status,
            message: None,
        }
    }

    /// Returns the status code of the error.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Returns the error message, if any.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Returns the status code and error message of this error.
    pub fn into_parts(self) -> (StatusCode, Option<String>) {
        (self.status, self.message)
    }
}

impl ResponseError {
    /// Returns a `400` bad request response error
    pub fn bad_request(error: impl Display) -> Self {
        ResponseError::new(StatusCode::BAD_REQUEST, error)
    }

    /// Returns a `401` unauthorized response error
    pub fn unauthorized(error: impl Display) -> Self {
        ResponseError::new(StatusCode::UNAUTHORIZED, error)
    }

    /// Returns a `403` forbidden response error
    pub fn forbidden(error: impl Display) -> Self {
        ResponseError::new(StatusCode::FORBIDDEN, error)
    }

    /// Returns a `404` not found response error
    pub fn not_found(error: impl Display) -> Self {
        ResponseError::new(StatusCode::NOT_FOUND, error)
    }

    /// Returns a `422` unprocessable entity response error
    pub fn unprocessable_entity(error: impl Display) -> Self {
        ResponseError::new(StatusCode::UNPROCESSABLE_ENTITY, error)
    }

    /// Returns a `500` internal server error response error
    pub fn internal_server_error(error: impl Display) -> Self {
        ResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, error)
    }
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.message {
            Some(message) => write!(f, "{message}"),
            None => match self.status.canonical_reason() {
                Some(reason) => write!(f, "{}", reason),
                None => write!(f, "Error {}, Something went wrong", self.status),
            },
        }
    }
}

impl std::error::Error for ResponseError {}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        #[derive(Serialize, Deserialize)]
        struct ErrorMessage {
            message: String,
        }

        let this = self.clone();
        let (status, message) = self.into_parts();

        let mut res = match message {
            Some(message) => {
                let json = Json(ErrorMessage { message });
                (status, json).into_response()
            }
            None => status.into_response(),
        };

        // We also insert the error as an extension in the response
        res.extensions_mut().insert(this);
        res
    }
}

// ResponseError is available for error handlers if an error ocurred,
// checkout `AppService::handle_error`
impl FromRequest for ResponseError {
    type Error = Error;
    type Fut = Ready<Result<ResponseError, Error>>;

    fn from_request(ctx: &crate::app::RequestContext) -> Self::Fut {
        let err = ctx
            .request()
            .extensions()
            .get::<ResponseError>()
            .cloned()
            .ok_or_else(|| "request does not contains an error".to_owned().into());

        std::future::ready(err)
    }
}

impl From<StatusCode> for ResponseError {
    fn from(status: StatusCode) -> Self {
        ResponseError {
            status,
            message: None,
        }
    }
}

impl From<(StatusCode, Option<String>)> for ResponseError {
    fn from((status, message): (StatusCode, Option<String>)) -> Self {
        ResponseError { status, message }
    }
}

impl<'a> From<(StatusCode, Option<&'a str>)> for ResponseError {
    fn from((status, message): (StatusCode, Option<&'a str>)) -> Self {
        (status, message.to_owned()).into()
    }
}

impl From<(StatusCode, String)> for ResponseError {
    fn from((status, message): (StatusCode, String)) -> Self {
        (status, Some(message)).into()
    }
}

impl<'a> From<(StatusCode, &'a str)> for ResponseError {
    fn from((status, message): (StatusCode, &'a str)) -> Self {
        (status, Some(message)).into()
    }
}
