use crate::{
    types::BoxFuture,
    web::{Body, Request, Response},
};
use futures::Future;

/// Resolves the next request and return the response.
pub type Next = Box<dyn FnOnce(Request<()>, Body) -> BoxFuture<Response> + Send + Sync>;

#[doc(hidden)]
pub trait OnHandleClone {
    fn clone_handler(&self) -> Box<dyn OnHandle + Send + Sync>;
}

impl<T> OnHandleClone for T
where
    T: Clone + OnHandle + Send + Sync + 'static,
{
    fn clone_handler(&self) -> Box<dyn OnHandle + Send + Sync> {
        Box::new(self.clone())
    }
}

/// A hook to the application request handler.
#[async_trait::async_trait]
pub trait OnHandle: OnHandleClone {
    /// Called on the next request.
    async fn call(&self, req: Request<()>, body: Body, next: Next) -> Response;
}

#[async_trait::async_trait]
impl<F, Fut> OnHandle for F
where
    F: Fn(Request<()>, Body, Next) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    async fn call(&self, req: Request<()>, body: Body, next: Next) -> Response {
        (self)(req, body, next).await
    }
}
