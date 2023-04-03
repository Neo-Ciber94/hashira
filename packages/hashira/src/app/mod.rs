mod app;
mod app_service;
mod context;
mod route;

pub use app::*;
pub use app_service::*;
pub use context::*;
pub use route::*;

//
pub mod client_router;
pub mod error_router;

// A convenient boxed future.
pub(crate) type BoxFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T>>>;
