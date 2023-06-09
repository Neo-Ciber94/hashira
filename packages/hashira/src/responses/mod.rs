use crate::{
    error::{BoxError, ServerError},
    web::IntoResponse,
};
use http::StatusCode;
use std::fmt::Display;

/// Creates a 400 bad request response error with the given message.
pub fn bad_request(msg: impl Display) -> BoxError {
    ServerError::new(StatusCode::BAD_REQUEST, msg).into()
}

/// Creates a 400 bad request response error with the error.
pub fn bad_request_with<T>(error: T) -> BoxError
where
    T: IntoResponse + Send + Sync + Clone + 'static,
{
    ServerError::from_response_and_status(StatusCode::BAD_REQUEST, error).into()
}

/// Creates a 401 unauthorized response error with the given message.
pub fn unauthorized(msg: impl Display) -> BoxError {
    ServerError::new(StatusCode::UNAUTHORIZED, msg).into()
}

/// Creates a 401 unauthorized response error with the error.
pub fn unauthorized_with<T>(error: T) -> BoxError
where
    T: IntoResponse + Send + Sync + Clone + 'static,
{
    ServerError::from_response_and_status(StatusCode::UNAUTHORIZED, error).into()
}

/// Creates a 403 forbidden response error with the given message.
pub fn forbidden(msg: impl Display) -> BoxError {
    ServerError::new(StatusCode::FORBIDDEN, msg).into()
}

/// Creates a 403 forbidden response error with the error.
pub fn forbidden_with<T>(error: T) -> BoxError
where
    T: IntoResponse + Send + Sync + Clone + 'static,
{
    ServerError::from_response_and_status(StatusCode::FORBIDDEN, error).into()
}

/// Creates a 404 not found response error with the given message.
pub fn not_found(msg: impl Display) -> BoxError {
    ServerError::new(StatusCode::NOT_FOUND, msg).into()
}

/// Creates a 404 not found response error with the error.
pub fn not_found_with<T>(error: T) -> BoxError
where
    T: IntoResponse + Send + Sync + Clone + 'static,
{
    ServerError::from_response_and_status(StatusCode::NOT_FOUND, error).into()
}

/// Creates a 405 method not allowed response error with the given message.
pub fn method_not_allowed(msg: impl Display) -> BoxError {
    ServerError::new(StatusCode::METHOD_NOT_ALLOWED, msg).into()
}

/// Creates a 405 method not allowed response error with the error.
pub fn method_not_allowed_with<T>(error: T) -> BoxError
where
    T: IntoResponse + Send + Sync + Clone + 'static,
{
    ServerError::from_response_and_status(StatusCode::METHOD_NOT_ALLOWED, error).into()
}

/// Creates a 409 conflict response error with the given message.
pub fn conflict(msg: impl Display) -> BoxError {
    ServerError::new(StatusCode::CONFLICT, msg).into()
}

/// Creates a 409 conflict response error with the error.
pub fn conflict_with<T>(error: T) -> BoxError
where
    T: IntoResponse + Send + Sync + Clone + 'static,
{
    ServerError::from_response_and_status(StatusCode::CONFLICT, error).into()
}

/// Creates a 422 unprocessable entity response error with the given message.
pub fn unprocessable_entity(msg: impl Display) -> BoxError {
    ServerError::new(StatusCode::UNPROCESSABLE_ENTITY, msg).into()
}

/// Creates a 422 unprocessable entity response error with the error.
pub fn unprocessable_entity_with<T>(error: T) -> BoxError
where
    T: IntoResponse + Send + Sync + Clone + 'static,
{
    ServerError::from_response_and_status(StatusCode::UNPROCESSABLE_ENTITY, error).into()
}

/// Creates a 500 internal server error response error with the given message.
pub fn internal_server_error(msg: impl Display) -> BoxError {
    ServerError::new(StatusCode::INTERNAL_SERVER_ERROR, msg).into()
}

/// Creates a 500 internal server error response error with the error.
pub fn internal_server_error_with<T>(error: T) -> BoxError
where
    T: IntoResponse + Send + Sync + Clone + 'static,
{
    ServerError::from_response_and_status(StatusCode::INTERNAL_SERVER_ERROR, error).into()
}
