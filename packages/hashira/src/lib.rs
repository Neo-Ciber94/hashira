/// Server adapter.
pub mod adapter;

/// Entry point for a `hashira` application.
pub mod app;

/// Common components.
pub mod components;

/// Environment variables.
pub mod env;

/// Base error type.
pub mod error;

/// Functional component hooks.
pub mod hooks;

/// Base routing.
pub mod routing;

/// Server related. 
pub mod server;
pub mod web;

// Allow public?
pub(crate) mod context;

/// Client related.
#[cfg(feature = "client")]
pub mod client;

/// A collection of event `hooks` of hashira.
#[cfg(feature = "hooks")]
pub mod events;

pub(crate) mod types;

/// Macro attribute for declaring [`PageComponent`]s.
/// 
/// [`PageComponent`]: ./components/trait.PageComponent.html
pub use hashira_macros::page_component;

mod reexports {
    /// Reexport of `async_trait`
    pub use async_trait::async_trait;
}

pub use reexports::*;

/// Constants.
pub mod consts {
    /// A constants indicating whether if the application is running on the server.
    #[cfg(not(feature = "client"))]
    pub const IS_SERVER: bool = true;

    /// A constants indicating whether if the application is running on the server.
    #[cfg(feature = "client")]
    pub const IS_SERVER: bool = false;
}
