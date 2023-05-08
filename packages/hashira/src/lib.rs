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
pub type Result<T> = std::result::Result<T, crate::error::Error>;

/// Macro attribute for declaring [`PageComponent`]s.
///
/// [`PageComponent`]: ./components/trait.PageComponent.html
pub use hashira_macros::*;

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

/// Extracts the `Ok(x)` value from a result, otherwise return an error `Response`.
#[macro_export]
macro_rules! try_response {
    ($result:expr) => {
        match $result {
            Ok(res) => res,
            Err(err) => {
                return $crate::web::IntoResponse::into_response(
                    $crate::error::ResponseError::with_error(err),
                );
            }
        }
    };
}

/// Client side utilities.
pub mod utils {

    // FIXME: When we implement the client routing this should be done with a router hook.
    /// Redirects to the given route.
    #[allow(unused_variables)]
    pub fn redirect_to<S: Into<String>>(route: S) {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::window;

            let window = window().unwrap();
            let location = window.location();
            location.assign(&route.into()).unwrap();
        }
    }

    /// Shows a message in the browser.
    #[allow(unused_variables)]
    pub fn show_alert<S: Into<String>>(message: S) {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::window;

            let window = window().unwrap();
            window.alert_with_message(&message.into()).unwrap();
        }
    }

    /// Reload the current page.
    #[allow(unused_variables)]
    pub fn refresh_window() {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::window;

            let window = window().unwrap();
            let location = window.location();
            location.reload().unwrap();
        }
    }
}
