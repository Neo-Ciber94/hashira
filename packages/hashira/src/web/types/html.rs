use http::header;
use crate::web::{Body, IntoResponse, Response};

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
