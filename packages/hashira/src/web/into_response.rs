use super::{extensions::ErrorMessage, Response, ResponseExt};
use crate::{app::ResponseError, error::Error};
use http::{header, StatusCode};

/// Convert an object into a `Response`.
pub trait IntoResponse {
    /// Converts this object into a response.
    fn into_response(self) -> Response;
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response::text(self)
    }
}

impl<'a> IntoResponse for &'a str {
    fn into_response(self) -> Response {
        Response::text(self)
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        Response::with_status(self)
    }
}

impl<'a> IntoResponse for (StatusCode, &'a str) {
    fn into_response(self) -> Response {
        let (status, body) = self;
        (status, body.to_owned()).into_response()
    }
}

impl IntoResponse for (StatusCode, String) {
    fn into_response(self) -> Response {
        let (status, body) = self;
        let mut res = Response::with_status(status);
        *res.body_mut() = body.into();
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        res
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl<T: IntoResponse> IntoResponse for Option<T> {
    fn into_response(self) -> Response {
        match self {
            Some(x) => x.into_response(),
            None => Response::with_status(StatusCode::NOT_FOUND),
        }
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: Into<Error>,
{
    fn into_response(self) -> Response {
        match self {
            Ok(x) => x.into_response(),
            Err(err) => ResponseError::from_error(err).into_response(),
        }
    }
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        let (status, message) = self.into_parts();
        match message {
            Some(msg) => {
                let error_message = ErrorMessage(msg.clone());
                let mut res = (status, msg).into_response();
                res.extensions_mut().insert(error_message);
                res
            }
            None => status.into_response(),
        }
    }
}
