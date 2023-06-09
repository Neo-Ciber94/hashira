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

/// Web types.
pub mod web;

/// Server actions.
pub mod actions;

/// Helpers for responses.
pub mod responses;

// Allow public?
pub(crate) mod context;

/// Client related.
#[cfg(feature = "client")]
pub mod client;

/// A collection of event `hooks` of hashira.
#[cfg(feature = "hooks")]
pub mod events;

#[doc(hidden)]
pub mod types;

/// A result type.
pub type Result<T> = std::result::Result<T, crate::error::BoxError>;

/// Macro attribute for declaring [`PageComponent`]s.
///
/// [`PageComponent`]: ./components/trait.PageComponent.html
pub use hashira_macros::*;

mod reexports {
    /// Reexport of `async_trait`
    pub use async_trait::async_trait;

    pub use multer_derive;
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

// Yeah, you are not suppose to use this
#[doc(hidden)]
#[cfg(feature = "internal")]
pub mod internal;

#[doc(hidden)]
pub mod utils;

/// Extracts the `Ok(x)` value from a result, otherwise return an error `Response`.
#[doc(hidden)]
#[macro_export]
macro_rules! try_response {
    ($result:expr) => {
        match $result {
            Ok(res) => res,
            Err(err) => {
                return $crate::web::IntoResponse::into_response(
                    $crate::error::ServerError::from_error(err),
                );
            }
        }
    };
}
