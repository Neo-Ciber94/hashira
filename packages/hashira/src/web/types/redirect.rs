use http::{StatusCode, uri::InvalidUri, header, Uri};
use crate::web::{IntoResponse, Response, Body};

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
