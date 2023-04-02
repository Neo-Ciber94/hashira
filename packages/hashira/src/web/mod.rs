
pub type Body = bytes::Bytes;

pub type Request<T = Body> = http::request::Request<T>;

pub type Response<T = Body> = http::response::Response<T>;

pub use http::header;
pub use http::method;
pub use http::status;
pub use http::uri;
pub use http::version;
pub use http::Error;
pub use http::Extensions;
