use std::fmt::{Debug, Display};

use super::Error;
use crate::web::{IntoResponse, Response, ResponseExt};
use http::StatusCode;
use thiserror::Error;

/// An error produced when a status code is not an error.
#[derive(Debug, Error)]
#[non_exhaustive]
#[error("`{status}` is not a valid error status code")]
pub struct InvalidErrorStatusCode {
    pub status: StatusCode,
}

enum Responder {
    Message(String),
    Response(Response),
    Factory(Box<dyn Fn() -> Response + Send + Sync>),
}

/// An error that ocurred in the server.
pub struct ServerError {
    status: StatusCode,
    responder: Option<Responder>,
}

impl ServerError {
    /// Construct a new error.
    pub fn new(status: StatusCode, msg: impl Display) -> Result<Self, InvalidErrorStatusCode> {
        Ok(ServerError {
            status: assert_status_code(status)?,
            responder: Some(Responder::Message(msg.to_string())),
        })
    }

    /// Constructs a new error from a response.
    pub fn from_response(response: Response) -> Result<Self, InvalidErrorStatusCode> {
        Ok(ServerError {
            status: assert_status_code(response.status())?,
            responder: Some(Responder::Response(response)),
        })
    }

    /// Construct a new error from the given type that implements `IntoResponse`.
    pub fn from_factory<R>(status: StatusCode, res: R) -> Result<Self, InvalidErrorStatusCode>
    where
        R: IntoResponse + Clone + Send + Sync + 'static,
    {
        Ok(ServerError {
            status: assert_status_code(status)?,
            responder: Some(Responder::Factory(Box::new(move || {
                res.clone().into_response()
            }))),
        })
    }

    /// Constructs a new error from a status code.
    pub fn from_status(status: StatusCode) -> Result<Self, InvalidErrorStatusCode> {
        Ok(ServerError {
            status: assert_status_code(status)?,
            responder: None,
        })
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
                Responder::Factory(_) => None,
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
                Responder::Response(_) => None,
                Responder::Factory(_) => {
                    let res = self.as_response();
                    let content_type = res.content_type()?;
                    if content_type.essence_str() == mime::TEXT_PLAIN.essence_str() {
                        res.body()
                            .try_as_bytes()
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

    /// Returns a response for this error.
    pub fn as_response(&self) -> Response {
        match &self.responder {
            Some(responder) => {
                let mut res = match responder {
                    Responder::Message(msg) => msg.clone().into_response(),
                    Responder::Response(res) => res.status().into_response(),
                    Responder::Factory(factory) => factory(),
                };

                *res.status_mut() = self.status();
                res
            }
            None => self.status.into_response(),
        }
    }
}

impl From<StatusCode> for ServerError {
    fn from(status: StatusCode) -> Self {
        ServerError::from_status(status).expect("invalid status code")
    }
}

impl From<(StatusCode, String)> for ServerError {
    fn from((status, msg): (StatusCode, String)) -> Self {
        ServerError::new(status, msg).expect("invalid status code")
    }
}

impl std::error::Error for ServerError {}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        if let Some(Responder::Response(res)) = self.responder {
            return res;
        }

        self.as_response()
    }
}

impl Debug for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.responder {
            Some(res) => match res {
                Responder::Message(msg) => write!(f, "{:?}", msg),
                Responder::Factory(_) => write!(f, "{:?}", self.status),
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
                Responder::Factory(_) => write!(f, "{}", self.status),
                Responder::Response(_) => write!(f, "{:?}", self.status),
            },
            None => write!(f, "{}", self.status),
        }
    }
}

fn assert_status_code(status: StatusCode) -> Result<StatusCode, InvalidErrorStatusCode> {
    if !(status.is_client_error() || status.is_server_error()) {
        return Err(InvalidErrorStatusCode { status });
    }

    Ok(status)
}
