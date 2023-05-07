use super::{Body, Response, ResponseExt};
use crate::{app::ResponseError, error::Error, types::TryBoxStream};
use bytes::Bytes;
use cookie::Cookie;
use futures::Stream;
use http::{header, HeaderMap, StatusCode};

/// Convert an object into a `Response`.
pub trait IntoResponse {
    /// Converts this object into a response.
    fn into_response(self) -> Response;
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        let body = Body::from(self);
        let mut res = Response::new(body);
        res.headers_mut().append(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        res
    }
}

impl<'a> IntoResponse for &'a str {
    fn into_response(self) -> Response {
        self.to_owned().into_response()
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        let body = Body::from(self);
        let mut res = Response::new(body);
        res.headers_mut().append(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/octet-stream"),
        );
        res
    }
}

impl<const N: usize> IntoResponse for [u8; N] {
    fn into_response(self) -> Response {
        self.to_vec().into_response()
    }
}

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> Response {
        let body = Body::from(self);
        let mut res = Response::new(body);
        res.headers_mut().append(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/octet-stream"),
        );
        res
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        Response::with_status(self, Body::default())
    }
}

impl<T> IntoResponse for (StatusCode, T)
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        let (status, rest) = self;
        let mut res = rest.into_response();
        *res.status_mut() = status;
        res
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::default()
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
            None => Response::with_status(StatusCode::NOT_FOUND, Body::default()),
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
            Err(err) => ResponseError::with_error(err).into_response(),
        }
    }
}

impl IntoResponse for serde_json::Value {
    fn into_response(self) -> Response {
        let body = Body::from(self.to_string());
        let mut res = Response::new(body);
        res.headers_mut().append(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        res
    }
}

impl<'a> IntoResponse for Cookie<'a> {
    fn into_response(self) -> Response {
        let mut response = Response::default();
        let cookie_str = self.to_string();
        response
            .headers_mut()
            .insert(header::SET_COOKIE, cookie_str.parse().unwrap());
        response
    }
}

impl<'a> IntoResponse for Vec<Cookie<'a>> {
    fn into_response(self) -> Response {
        let mut response = Response::default();
        for cookie in self {
            let cookie_str = cookie.to_string();
            response
                .headers_mut()
                .append(header::SET_COOKIE, cookie_str.parse().unwrap());
        }
        response
    }
}

impl IntoResponse for HeaderMap {
    fn into_response(self) -> Response {
        let mut response = Response::default();

        let mut last_header_name: Option<header::HeaderName> = None;
        for (header_name, header_value) in self.into_iter() {
            // SAFETY: A valid header name must be emitted before any `None` header
            let header_name = header_name.unwrap_or_else(|| last_header_name.unwrap());
            response
                .headers_mut()
                .append(header_name.clone(), header_value);
            last_header_name = Some(header_name);
        }

        response
    }
}

/// Represents a streamed response.
pub struct StreamResponse<S>(pub S);
impl<S> IntoResponse for StreamResponse<S>
where
    S: Stream<Item = Result<Bytes, Error>> + Send + Sync + 'static,
{
    fn into_response(self) -> Response {
        let stream = Box::pin(self.0) as TryBoxStream<Bytes>;
        let body = Body::from(stream);
        Response::builder()
            .header(header::TRANSFER_ENCODING, "chunked")
            .body(body)
            .unwrap()
    }
}
