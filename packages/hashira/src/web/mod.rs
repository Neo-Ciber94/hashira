#[doc(hidden)]
pub mod serde;

mod body;
mod from_request;
mod into_json;
mod into_response;
mod request_ext;
mod response_ext;
mod types;

pub use from_request::*;
pub use into_json::*;
pub use into_response::*;
pub use request_ext::*;
pub use response_ext::*;
pub use types::*;

pub use bytes::{Bytes, BytesMut};

/// Represents a `http` request.
pub type Request<T = Body> = http::request::Request<T>;

/// Represents a `http` response.
pub type Response<T = Body> = http::response::Response<T>;

pub use body::*;
pub use bytes::*;
pub use cookie::*;
pub use http::header;
pub use http::method;
pub use http::status;
pub use http::uri;
pub use http::version;
pub use http::Error;
pub use http::Extensions;
