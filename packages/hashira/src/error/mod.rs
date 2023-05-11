mod js_error;
pub use js_error::*;

mod server_error;
pub use server_error::*;

/// A boxed error.
pub type Error = Box<dyn std::error::Error + Send + Sync>;
