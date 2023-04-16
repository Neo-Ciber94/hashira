#[doc(hidden)]
pub mod serde;

mod into_response;
mod request_ext;
mod response_ext;
mod body;

pub use into_response::*;
pub use request_ext::*;
pub use response_ext::*;

/// Represents a `http` request.
pub type Request<T = Body> = http::request::Request<T>;

/// Represents a `http` response.
pub type Response<T = Body> = http::response::Response<T>;

pub use http::header;
pub use http::method;
pub use http::status;
pub use http::uri;
pub use http::version;
pub use http::Error;
pub use http::Extensions;
pub use body::*;
