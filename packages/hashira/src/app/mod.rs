mod app;
mod app_scope;
mod app_service;
mod layout_context;
mod layout_data;
mod render_context;
mod request_context;
mod route;

pub use app::*;
pub use app_scope::*;
pub use app_service::*;
pub use layout_context::*;
pub use render_context::*;
pub use request_context::*;
pub use route::*;

//
pub mod error_router;
pub mod router;

// A convenient boxed future.
pub(crate) type BoxFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T>>>;
