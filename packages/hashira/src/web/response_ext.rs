use std::str::FromStr;

use super::Response;
use cookie::Cookie;
use http::{
    header::{self, InvalidHeaderValue, COOKIE, SET_COOKIE},
    HeaderValue, StatusCode,
};

/// Extension methods for `Response`.
pub trait ResponseExt<B> {
    /// Creates a response with the given status code and body.
    fn with_status(status: StatusCode, body: B) -> Response<B>;

    // TODO: Return an iterator instead.
    /// Returns all the cookies in the request.
    fn cookies(&self) -> Result<Vec<Cookie<'static>>, cookie::ParseError>;

    /// Sets a `Cookie`.
    fn set_cookie(&mut self, cookie: Cookie<'static>) -> Result<(), InvalidHeaderValue>;

    /// Remove all the cookies in the current request with the given name.
    fn del_cookie(&mut self, name: &str) -> usize;

    /// Returns the content type of this response.
    fn content_type(&self) -> Option<mime::Mime>;
}

impl<B> ResponseExt<B> for Response<B> {
    fn with_status(status: StatusCode, body: B) -> Response<B> {
        let mut res = Response::new(body);
        *res.status_mut() = status;
        res
    }

    fn cookies(&self) -> Result<Vec<Cookie<'static>>, cookie::ParseError> {
        // Adapted from: https://docs.rs/actix-web/latest/src/actix_web/request.rs.html#315-334

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

    fn content_type(&self) -> Option<mime::Mime> {
        self.headers()
            .get(header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .and_then(|x| mime::Mime::from_str(x).ok())
    }
}
