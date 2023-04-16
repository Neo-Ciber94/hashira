use std::sync::Arc;

use crate::{web::{Request, Response}, types::BoxFuture};

/// Resolves the next request and return the response.
pub type Next = Box<dyn FnOnce(Arc<Request>) -> BoxFuture<Response> + Send + Sync>;

/// A hook to the application request handler.
#[async_trait::async_trait]
pub trait OnHandle {
    /// Called on the next request.
    async fn on_handle(&self, req: Arc<Request>, next: Next) -> Response;
}

#[async_trait::async_trait]
impl<F> OnHandle for F
where
    F: Fn(Arc<Request>, Next) -> Response + Send + Sync + 'static,
{
    async fn on_handle(&self, req: Arc<Request>, next: Next) -> Response {
        (self)(req, next)
    }
}
