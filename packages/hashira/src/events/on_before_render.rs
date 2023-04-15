use crate::app::RequestContext;

/// A hook to the application before render event.
#[async_trait::async_trait]
pub trait OnBeforeRender {
    /// Called before render.
    async fn on_before_render(&self, html: &mut String, ctx: &RequestContext);
}

#[async_trait::async_trait]
impl<F> OnBeforeRender for F
where
    F: Fn(&mut String, &RequestContext) + Send + Sync + 'static,
{
    async fn on_before_render(&self, html: &mut String, ctx: &RequestContext) {
        (self)(html, ctx);
    }
}