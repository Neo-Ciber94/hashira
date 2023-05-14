use super::Error;
use crate::web::{IntoResponse, Response, ResponseExt};
use http::StatusCode;
use std::fmt::{Debug, Display};

enum Responder {
    Message(String),
    Response(Response),
}

/// An error that ocurred in the server.
pub struct ServerError {
    status: StatusCode,
    responder: Option<Responder>,
}

impl ServerError {
    /// Construct a new error.
    pub fn new(status: StatusCode, msg: impl Display) -> Self {
        assert_status_code(status);

        ServerError {
            status,
            responder: Some(Responder::Message(msg.to_string())),
        }
    }

    fn _from_response(status: StatusCode, response: Response) -> Self {
        assert_status_code(status);
        let res = response.into_response();

        ServerError {
            status,
            responder: Some(Responder::Response(res)),
        }
    }

    /// Constructs a new error from a response.
    pub fn from_response<T: IntoResponse>(response: T) -> Self {
        let response = response.into_response();
        Self::_from_response(response.status(), response)
    }

    /// Constructs a new error from a response with the given status,
    pub fn from_response_and_status<T: IntoResponse>(status: StatusCode, response: T) -> Self {
        let response = response.into_response();
        Self::_from_response(status, response)
    }

    /// Constructs a new error from a status code.
    pub fn from_status(status: StatusCode) -> Self {
        assert_status_code(status);
        ServerError {
            status,
            responder: None,
        }
    }

    /// Constructs a new error from other.
    pub fn from_error(error: impl Into<Error>) -> Self {
        let error = error.into();

        if error.is::<ServerError>() {
            return *error.downcast().unwrap();
        }

        let msg = error.to_string();
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            responder: Some(Responder::Message(msg)),
        }
    }

    /// Returns the status code.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Returns the message of this error, if any.
    pub fn message(&self) -> Option<&str> {
        match &self.responder {
            Some(res) => match res {
                Responder::Message(msg) => Some(msg.as_str()),
                Responder::Response(_) => None,
            },
            None => None,
        }
    }

    // Attempts to get the error message from the error or the response.
    #[allow(dead_code)]
    pub(crate) async fn try_get_message(&self) -> Option<String> {
        match &self.responder {
            Some(res) => match res {
                Responder::Message(msg) => Some(msg.to_owned()),
                Responder::Response(res) => {
                    let content_type = res.content_type()?;
                    if content_type.essence_str() == mime::TEXT_PLAIN.essence_str() {
                        res.body()
                            .try_to_bytes()
                            .ok()
                            .and_then(|x| String::from_utf8(x.to_vec()).ok())
                    } else {
                        None
                    }
                }
            },
            None => None,
        }
    }
}

impl From<Response> for ServerError {
    fn from(response: Response) -> Self {
        ServerError::from_response(response)
    }
}

impl From<StatusCode> for ServerError {
    fn from(status: StatusCode) -> Self {
        ServerError::from_status(status)
    }
}

impl From<(StatusCode, String)> for ServerError {
    fn from((status, msg): (StatusCode, String)) -> Self {
        ServerError::new(status, msg)
    }
}

impl std::error::Error for ServerError {}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = self.status();
        match self.responder {
            Some(responder) => {
                let mut res = match responder {
                    Responder::Message(msg) => msg.into_response(),
                    Responder::Response(res) => res,
                };

                *res.status_mut() = status;
                res
            }
            None => status.into_response(),
        }
    }
}

impl Debug for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.responder {
            Some(res) => match res {
                Responder::Message(msg) => write!(f, "{:?}", msg),
                Responder::Response(_) => write!(f, "{:?}", self.status),
            },
            None => write!(f, "{:?}", self.status),
        }
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.responder {
            Some(res) => match res {
                Responder::Message(msg) => write!(f, "{}", msg),
                Responder::Response(_) => write!(f, "{:?}", self.status),
            },
            None => write!(f, "{}", self.status),
        }
    }
}

fn assert_status_code(status: StatusCode) {
    if !(status.is_client_error() || status.is_server_error()) {
        panic!("`{status}` is not a valid error status code");
    }
}
