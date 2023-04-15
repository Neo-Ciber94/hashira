/// A hook to the application when rendering a chunk of html,
/// this may be a chunk of a HTML being streamed or after the complete html had been rendered.
pub trait OnChunkRender {
    /// Called when rendering a chunk of html.
    fn on_chunk_render(&self, html: &mut String);
}

impl<F> OnChunkRender for F
where
    F: Fn(&mut String) + Send + Sync + 'static,
{
    fn on_chunk_render(&self, html: &mut String) {
        (self)(html);
    }
}
