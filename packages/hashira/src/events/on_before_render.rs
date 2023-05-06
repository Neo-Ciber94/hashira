use futures::Future;

use crate::{app::RequestContext, error::Error, types::BoxFuture};

/// A hook to the application before render event.
pub trait OnBeforeRender: Sync {
    type Fut: Future<Output = Result<String, Error>> + Send;
    /// Called before render.
    fn call(&self, html: String, ctx: RequestContext) -> Self::Fut;
}

impl<F, Fut> OnBeforeRender for F
where
    Fut: Future<Output = Result<String, Error>> + Send + 'static,
    F: Fn(String, RequestContext) -> Fut + Send + Sync + 'static,
{
    type Fut = BoxFuture<Result<String, Error>>;

    fn call(&self, html: String, ctx: RequestContext) -> Self::Fut {
        Box::pin((self)(html, ctx))
    }
}

#[allow(clippy::type_complexity)]
pub struct BoxOnBeforeRender(
    Box<dyn Fn(String, RequestContext) -> BoxFuture<Result<String, Error>> + Send + Sync>,
);

impl BoxOnBeforeRender {
    pub fn new<F>(f: F) -> Self
    where
        F: OnBeforeRender + Send + Sync + 'static,
        F::Fut: Send + 'static,
    {
        let inner = Box::new(move |html, ctx: RequestContext| {
            let fut = f.call(html, ctx);
            Box::pin(fut) as BoxFuture<Result<String, Error>>
        });

        BoxOnBeforeRender(inner)
    }
}

impl OnBeforeRender for BoxOnBeforeRender {
    type Fut = BoxFuture<Result<String, Error>>;

    fn call(&self, html: String, ctx: RequestContext) -> Self::Fut {
        (self.0)(html, ctx)
    }
}
