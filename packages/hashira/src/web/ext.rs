use std::fmt::Display;

use bytes::Bytes;
use http::{
    header::{InvalidHeaderValue, CONTENT_TYPE, LOCATION},
    HeaderValue, StatusCode,
};
use serde::Serialize;

use super::{Body, Response};

impl std::error::Error for RedirectionError {}

#[derive(Debug)]
pub enum RedirectionError {
    InvalidStatus(StatusCode),
    InvalidHeader(InvalidHeaderValue),
}

impl Display for RedirectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RedirectionError::InvalidStatus(status) => {
                write!(f, "{} is not a valid redirection status code", status)
            }
            RedirectionError::InvalidHeader(err) => write!(f, "{err}"),
        }
    }
}

/// Extension methods for `Response`.
pub trait ResponseExt {
    /// Create a `text/html` response.
    fn html(body: impl Into<Bytes>) -> Response;

    /// Creates a `application/json` response.
    fn json<T>(data: T) -> Result<Response, serde_json::Error>
    where
        T: Serialize;

    /// Creates a redirection response to the given location.
    fn redirect(
        location: impl Into<String>,
        status: StatusCode,
    ) -> Result<Response, RedirectionError>;

    /// Creates a response with the given status code.
    fn status(status: StatusCode) -> Response;
}

impl ResponseExt for Response {
    fn html(body: impl Into<Bytes>) -> Response {
        let mut res = Response::new(body.into());
        res.headers_mut().append(
            CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        res
    }

    fn json<T>(data: T) -> Result<Response, serde_json::Error>
    where
        T: Serialize,
    {
        let json = serde_json::to_string(&data)?;
        let body = Body::from(json);
        let mut res = Response::new(body);
        res.headers_mut()
            .append(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(res)
    }

    fn redirect(
        location: impl Into<String>,
        status: StatusCode,
    ) -> Result<Response, RedirectionError> {
        if !status.is_redirection() {
            return Err(RedirectionError::InvalidStatus(status));
        }

        let location =
            HeaderValue::try_from(location.into()).map_err(RedirectionError::InvalidHeader)?;
        let mut res = Response::new(Body::new());
        res.headers_mut().append(LOCATION, location);
        Ok(res)
    }

    fn status(status: StatusCode) -> Response {
        let mut res = Response::new(Body::new());
        *res.status_mut() = status;
        res
    }
}
