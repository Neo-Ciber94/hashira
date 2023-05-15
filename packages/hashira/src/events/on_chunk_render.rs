use crate::{app::RequestContext, error::BoxError};

/// A hook to the application when rendering a chunk of html,
/// this may be a chunk of a HTML being streamed or after the complete html had been rendered.
pub trait OnChunkRender {
    /// Called when rendering a chunk of html.
    fn call(&self, chunk: String, ctx: RequestContext) -> Result<String, BoxError>;
}

impl<F> OnChunkRender for F
where
    F: Fn(String, RequestContext) -> Result<String, BoxError> + Send + Sync + 'static,
{
    fn call(&self, chunk: String, ctx: RequestContext) -> Result<String, BoxError> {
        (self)(chunk, ctx)
    }
}
