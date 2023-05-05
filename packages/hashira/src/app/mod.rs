mod action;
#[allow(clippy::module_inception)]
mod app;
mod app_data;
mod app_nested;
mod app_service;
mod default_headers;
mod handler;
mod layout_context;
mod render_context;
mod request_context;
mod route;

pub use action::handler::call_action;
pub use action::*;
pub use app::*;
pub use app_data::*;
pub use app_nested::*;
pub use app_service::*;
pub use default_headers::*;
pub use handler::*;
pub use layout_context::*;
pub use render_context::*;
pub use request_context::*;
pub use route::*;

//
pub mod error_router;
pub mod router;

//
pub(crate) mod page_head;
