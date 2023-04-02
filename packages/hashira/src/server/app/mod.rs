mod app;
mod app_service;
mod route;

pub use app::*;
pub use app_service::*;
pub use route::*;

pub(crate) type BoxFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T>>>;
