mod on_after_render;
mod on_before_render;
mod on_chunk_render;
mod on_client_error;
mod on_client_init;
mod on_handle;
mod on_server_error;
mod on_server_init;

pub use {
    on_after_render::*, on_before_render::*, on_chunk_render::*, on_client_error::*,
    on_client_init::*, on_handle::*, on_server_error::*, on_server_init::*,
};

use std::{fmt::Display, sync::Arc};
/// Represents a collection of event hooks.
#[derive(Default)]
pub struct Hooks {
    pub(crate) on_handle_hooks: Vec<Arc<dyn OnHandle + Send + Sync>>,
    pub(crate) on_before_render_hooks: Vec<Box<dyn OnBeforeRender + Send + Sync>>,
    pub(crate) on_after_render_hooks: Vec<Box<dyn OnAfterRender + Send + Sync>>,
    pub(crate) on_chunk_render_hooks: Vec<Box<dyn OnChunkRender + Send + Sync>>,
    pub(crate) on_server_initialize_hooks: Vec<Box<dyn OnServerInitialize + Send + Sync>>,
    pub(crate) on_client_initialize_hooks: Vec<Box<dyn OnClientInitialize + Send + Sync>>,
    pub(crate) on_server_error_hooks: Vec<Box<dyn OnServerError + Send + Sync>>,
    pub(crate) on_client_error_hooks: Vec<Box<dyn OnClientError + Send + Sync>>,
}

impl Hooks {
    /// Constructs an empty collection of hooks.
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a hook to be executed before handling a request.
    pub fn on_handle<F>(mut self, f: F) -> Self
    where
        F: OnHandle + Send + Sync + 'static,
    {
        self.on_handle_hooks.push(Arc::new(f));
        self
    }

    /// Adds a hook to be executed before rendering a response.
    pub fn on_before_render<F>(mut self, f: F) -> Self
    where
        F: OnBeforeRender + Send + Sync + 'static,
    {
        self.on_before_render_hooks.push(Box::new(f));
        self
    }

    /// Adds a hook to be executed after rendering a response.
    pub fn on_after_render<F>(mut self, f: F) -> Self
    where
        F: OnAfterRender + Send + Sync + 'static,
    {
        self.on_after_render_hooks.push(Box::new(f));
        self
    }

    /// Adds a hook to be executed after rendering a chunk of a response.
    pub fn on_chunk_render<F>(mut self, f: F) -> Self
    where
        F: OnChunkRender + Send + Sync + 'static,
    {
        self.on_chunk_render_hooks.push(Box::new(f));
        self
    }

    /// Adds a hook to be executed when the server is initialized.
    pub fn on_server_initialize<F>(mut self, f: F) -> Self
    where
        F: OnServerInitialize + Send + Sync + 'static,
    {
        self.on_server_initialize_hooks.push(Box::new(f));
        self
    }

    /// Adds a hook to be executed when the client is initialized.
    pub fn on_client_initialize<F>(mut self, f: F) -> Self
    where
        F: OnClientInitialize + Send + Sync + 'static,
    {
        self.on_client_initialize_hooks.push(Box::new(f));
        self
    }

    /// Adds a hook to be executed when a server error occurs.
    pub fn on_server_error<F>(mut self, f: F) -> Self
    where
        F: OnServerError + Send + Sync + 'static,
    {
        self.on_server_error_hooks.push(Box::new(f));
        self
    }

    /// Adds a hook to be executed when a client error occurs.
    pub fn on_client_error<F>(mut self, f: F) -> Self
    where
        F: OnClientError + Send + Sync + 'static,
    {
        self.on_client_error_hooks.push(Box::new(f));
        self
    }

    pub fn extend(&mut self, hooks: Hooks) {
        self.on_handle_hooks.extend(hooks.on_handle_hooks);

        self.on_before_render_hooks
            .extend(hooks.on_before_render_hooks);

        self.on_after_render_hooks
            .extend(hooks.on_after_render_hooks);

        self.on_chunk_render_hooks
            .extend(hooks.on_chunk_render_hooks);

        self.on_server_initialize_hooks
            .extend(hooks.on_server_initialize_hooks);

        self.on_client_initialize_hooks
            .extend(hooks.on_client_initialize_hooks);

        self.on_server_error_hooks
            .extend(hooks.on_server_error_hooks);

        self.on_client_error_hooks
            .extend(hooks.on_client_error_hooks);
    }
}

impl Display for Hooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hooks")
    }
}
