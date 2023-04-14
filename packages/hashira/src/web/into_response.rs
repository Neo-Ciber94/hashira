use super::{Body, Response, ResponseExt};
use crate::{app::ResponseError, error::Error};
use cookie::Cookie;
use http::{header, uri::InvalidUri, HeaderMap, StatusCode, Uri};
use serde::Serialize;
use std::convert::TryFrom;

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

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> Response {
        let body = Body::from_static(self);
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
            Err(err) => ResponseError::from_error(err).into_response(),
        }
    }
}

impl IntoResponse for serde_json::Value {
    fn into_response(self) -> Response {
        let body = Body::from(serde_json::to_string(&self).unwrap());
        let mut res = Response::new(body);
        res.headers_mut().append(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json; charset=utf-8"),
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
            response.headers_mut().append(header_name.clone(), header_value);
            last_header_name = Some(header_name);
        }

        response
    }
}

/// Represents a JSON.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Json<T>(pub T);
impl<T> Json<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        let json = match serde_json::to_string(&self.0) {
            Ok(s) => s,
            Err(err) => {
                return ResponseError::from_error(err).into_response();
            }
        };

        let body = Body::from(json);
        let mut res = Response::new(body);
        res.headers_mut().append(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json; charset=utf-8"),
        );
        res
    }
}

/// Represents a HTML.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Html<T>(pub T);
impl<T> Html<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: AsRef<str>> IntoResponse for Html<T> {
    fn into_response(self) -> Response {
        let body = Body::from(self.0.as_ref().to_owned());
        let mut res = Response::new(body);
        res.headers_mut().append(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("text/html; charset=utf-8"),
        );
        res
    }
}

/// Represents an HTTP redirect response.
#[derive(Debug, Clone)]
pub struct Redirect {
    uri: Uri,
    status: StatusCode,
}

impl Redirect {
    /// Create a new redirect response with the given URI and status code.
    pub fn new<U>(uri: U, status: StatusCode) -> Result<Self, InvalidUri>
    where
        Uri: TryFrom<U, Error = InvalidUri>,
    {
        let uri = Uri::try_from(uri)?;
        Ok(Redirect { uri, status })
    }

    /// Create a permanent redirect response with the given URI.
    pub fn permanent<U>(uri: U) -> Result<Self, InvalidUri>
    where
        Uri: TryFrom<U, Error = InvalidUri>,
    {
        Redirect::new(uri, StatusCode::MOVED_PERMANENTLY)
    }

    /// Create a temporary redirect response with the given URI.
    pub fn temporary<U>(uri: U) -> Result<Self, InvalidUri>
    where
        Uri: TryFrom<U, Error = InvalidUri>,
    {
        Redirect::new(uri, StatusCode::FOUND)
    }

    /// Create a see other redirect response with the given URI.
    pub fn see_other<U>(uri: U) -> Result<Self, InvalidUri>
    where
        Uri: TryFrom<U, Error = InvalidUri>,
    {
        Redirect::new(uri, StatusCode::SEE_OTHER)
    }
}

impl IntoResponse for Redirect {
    fn into_response(self) -> Response {
        let mut res = Response::new(Body::default());
        *res.status_mut() = self.status;
        res.headers_mut()
            .insert(header::LOCATION, self.uri.to_string().parse().unwrap());
        res
    }
}
