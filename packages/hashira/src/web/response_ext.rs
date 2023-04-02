use std::fmt::Display;
use bytes::Bytes;
use cookie::Cookie;
use http::{
    header::{InvalidHeaderValue, CONTENT_TYPE, COOKIE, LOCATION, SET_COOKIE},
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
    fn with_status(status: StatusCode) -> Response;

    // TODO: Return an iterator instead.
    /// Returns all the cookies in the request.
    fn cookies(&self) -> Result<Vec<Cookie>, cookie::ParseError>;

    /// Sets a `Cookie`.
    fn set_cookie(&mut self, cookie: Cookie) -> Result<(), InvalidHeaderValue>;

    /// Remove all the cookies in the current request with the given name.
    fn del_cookie(&mut self, name: &str) -> usize;
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

    fn with_status(status: StatusCode) -> Response {
        let mut res = Response::new(Body::new());
        *res.status_mut() = status;
        res
    }

    fn cookies(&self) -> Result<Vec<Cookie>, cookie::ParseError> {
        // Copied from: https://docs.rs/actix-web/latest/src/actix_web/request.rs.html#315-334

        let mut cookies = Vec::new();

        for header_value in self.headers().get_all(COOKIE) {
            let raw = std::str::from_utf8(header_value.as_bytes())
                .map_err(cookie::ParseError::Utf8Error)?;
            for cookie_str in raw.split(';').map(|s| s.trim()) {
                if !cookie_str.is_empty() {
                    cookies.push(Cookie::parse_encoded(cookie_str)?.into_owned());
                }
            }
        }

        Ok(cookies)
    }

    fn set_cookie(&mut self, cookie: Cookie) -> Result<(), InvalidHeaderValue> {
        HeaderValue::from_str(&cookie.to_string()).map(|cookie| {
            self.headers_mut().append(SET_COOKIE, cookie);
        })
    }

    fn del_cookie(&mut self, name: &str) -> usize {
        let headers = self.headers_mut();

        // Remove all the cookies
        headers.remove(SET_COOKIE);

        let values: Vec<HeaderValue> = headers
            .get_all(SET_COOKIE)
            .into_iter()
            .map(|v| v.to_owned())
            .collect();

        let mut removed_count = 0;

        for v in values {
            if let Ok(s) = v.to_str() {
                if let Ok(cookie) = Cookie::parse_encoded(s) {
                    if cookie.name() == name {
                        removed_count += 1;
                        continue;
                    }
                }
            }

            // Set the cookie back
            headers.append(SET_COOKIE, v);
        }

        removed_count
    }
}
