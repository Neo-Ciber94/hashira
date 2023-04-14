use super::Request;
use cookie::Cookie;
use http::header::COOKIE;
use serde::de::DeserializeOwned;
use thiserror::Error;

/// Error that occurred while trying to retrieve the query params.
#[derive(Error, Debug)]
pub enum QueryParamsError {
    /// No query params present in the url.
    #[error("not query string present in url")]
    NotFound,

    /// Failed to parse the query params.
    #[error(transparent)]
    Parse(serde_qs::Error),
}

/// Extension methods for `Request`.
pub trait RequestExt<B> {
    // TODO: Return an iterator instead.
    /// Get all the request cookies.
    fn cookies(&self) -> Result<Vec<Cookie>, cookie::ParseError>;

    /// Get the cookie with the given name.
    fn cookie(&self, name: &str) -> Option<Cookie>;

    /// Parses the query params of the url.
    fn query_params<Q: DeserializeOwned>(&self) -> Result<Q, QueryParamsError>;
}

impl<B> RequestExt<B> for Request<B> {
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

    fn cookie(&self, name: &str) -> Option<Cookie> {
        for hdr in self.headers().get_all(COOKIE) {
            let Ok(s) = std::str::from_utf8(hdr.as_bytes()) else {
                continue;
            };

            for cookie_str in s.split(';').map(|s| s.trim()) {
                if !cookie_str.is_empty() {
                    match Cookie::parse_encoded(cookie_str) {
                        Ok(cookie) if cookie.name() == name => return Some(cookie),
                        _ => {}
                    }
                }
            }
        }

        None
    }

    fn query_params<Q: DeserializeOwned>(&self) -> Result<Q, QueryParamsError> {
        let query_str = self
            .uri()
            .query()
            .ok_or(QueryParamsError::NotFound)?;

        serde_qs::from_str(query_str).map_err(QueryParamsError::Parse)
    }
}
