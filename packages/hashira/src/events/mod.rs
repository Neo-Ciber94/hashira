use crate::{
    app::{AppService, RequestContext, ResponseError, BoxFuture},
    web::{Request, Response},
};
pub use async_trait::*;
use std::{fmt::Display, panic::PanicInfo, sync::Arc};

/// Represents a collection of event hooks.
#[derive(Default)]
pub struct Hooks {
    pub(crate) on_handle_hooks: Vec<Box<dyn OnHandle + Send + Sync>>,
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
        self.on_handle_hooks.push(Box::new(f));
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

/// Resolves the next request and return the response.
pub type Next = Box<dyn FnOnce(Arc<Request>) -> BoxFuture<Response> + Send + Sync>;

/// A hook to the application request handler.
#[async_trait]
pub trait OnHandle {
    /// Called on the next request.
    async fn on_handle(&self, req: Arc<Request>, next: Next) -> Response;
}

/// A hook to the application before render event.
#[async_trait]
pub trait OnBeforeRender {
    /// Called before render.
    async fn on_before_render(&self, html: &mut String, ctx: &RequestContext);
}

/// A hook to the application after render event.
#[async_trait]
pub trait OnAfterRender {
    /// Called after render.
    async fn on_after_render(&self, html: &mut String, ctx: &RequestContext);
}

/// A hook to the application rendering when streaming the html.
pub trait OnChunkRender {
    /// Called while streaming a html chunk.
    fn on_chunk_render(&self, html: &mut String);
}

/// A hook called when the server is initialized.
pub trait OnServerInitialize {
    /// Called on server initialization.
    fn on_initialize(&self, service: &AppService);
}

/// A hook called on client initialization.
pub trait OnClientInitialize {
    /// Called on client initialization.
    fn on_initialize(&self, service: &AppService);
}

/// A hook called when an response error will be returned.
pub trait OnServerError {
    fn on_error(&self, err: &ResponseError);
}

/// A hook called when the wasm client panics.
pub trait OnClientError {
    /// Called on panics.
    fn on_error(&self, err: &PanicInfo);
}

#[async_trait]
impl<F> OnHandle for F
where
    F: Fn(Arc<Request>, Next) -> Response + Send + Sync + 'static,
{
    async fn on_handle(&self, req: Arc<Request>, next: Next) -> Response {
        (self)(req, next)
    }
}

#[async_trait]
impl<F> OnBeforeRender for F
where
    F: Fn(&mut String, &RequestContext) + Send + Sync + 'static,
{
    async fn on_before_render(&self, html: &mut String, ctx: &RequestContext) {
        (self)(html, ctx);
    }
}

#[async_trait]
impl<F> OnAfterRender for F
where
    F: Fn(&mut String, &RequestContext) + Send + Sync + 'static,
{
    async fn on_after_render(&self, html: &mut String, ctx: &RequestContext) {
        (self)(html, ctx);
    }
}

impl<F> OnChunkRender for F
where
    F: Fn(&mut String) + Send + Sync + 'static,
{
    fn on_chunk_render(&self, html: &mut String) {
        (self)(html);
    }
}

impl<F> OnServerInitialize for F
where
    F: Fn(&AppService) + Send + Sync + 'static,
{
    fn on_initialize(&self, service: &AppService) {
        (self)(service);
    }
}

impl<F> OnClientInitialize for F
where
    F: Fn(&AppService) + Send + Sync + 'static,
{
    fn on_initialize(&self, service: &AppService) {
        (self)(service);
    }
}

impl<F> OnServerError for F
where
    F: Fn(&ResponseError) + Send + Sync + 'static,
{
    fn on_error(&self, err: &ResponseError) {
        (self)(err)
    }
}

impl<F> OnClientError for F
where
    F: Fn(&PanicInfo) + Send + Sync + 'static,
{
    fn on_error(&self, err: &PanicInfo) {
        (self)(err)
    }
}
