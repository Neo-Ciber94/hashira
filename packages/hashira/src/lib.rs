pub mod app;
pub mod client;
pub mod components;
pub mod context;
pub mod env;
pub mod error;
pub mod hooks;
pub mod server;
pub mod web;
pub mod routing;


#[cfg(feature = "hooks")]
pub mod events;

pub(crate) mod types;

// Macros
pub use hashira_macros::*;

/// Constants.
pub mod consts {
    /// A constants indicating whether if the application is running on the server.
    #[cfg(not(target_arch = "wasm32"))]
    pub const IS_SERVER: bool = true;

    /// A constants indicating whether if the application is running on the server.
    #[cfg(target_arch = "wasm32")]
    pub const IS_SERVER: bool = false;
}
