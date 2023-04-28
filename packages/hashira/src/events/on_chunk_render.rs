use crate::{app::RequestContext, error::Error};

/// A hook to the application when rendering a chunk of html,
/// this may be a chunk of a HTML being streamed or after the complete html had been rendered.
pub trait OnChunkRender {
    /// Called when rendering a chunk of html.
    fn call(&self, chunk: &mut String, ctx: &RequestContext) -> Result<(), Error>;
}

impl<F> OnChunkRender for F
where
    F: Fn(&mut String, &RequestContext) -> Result<(), Error> + Send + Sync + 'static,
{
    fn call(&self, chunk: &mut String, ctx: &RequestContext) -> Result<(), Error> {
        (self)(chunk, ctx)
    }
}
