use http::StatusCode;
use std::fmt::Display;

/// A convenient error type.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A error that occurred while processing a request.
#[derive(Debug)]
pub struct ResponseError {
    status: StatusCode,
    message: Option<String>,
}

impl ResponseError {
    /// Constructs a new `ResponseError`.
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        ResponseError {
            status,
            message: Some(message.into()),
        }
    }

    /// Constructs a new `ResponseError` from an error.
    pub fn from_error<E: Into<Error>>(error: E) -> Self {
        let err = error.into();
        match err.downcast::<ResponseError>() {
            Ok(err) => *err,
            Err(err) => ResponseError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: Some(err.to_string()),
            },
        }
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

impl From<StatusCode> for ResponseError {
    fn from(status: StatusCode) -> Self {
        ResponseError {
            status,
            message: None,
        }
    }
}