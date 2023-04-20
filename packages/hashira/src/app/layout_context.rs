use super::{page_head::PageHead, RequestContext};
use crate::server::{Metadata, PageLinks, PageScripts};
use std::ops::Deref;

/// The context used to render the layout.
pub struct LayoutContext {
    context: RequestContext,
    head: PageHead,
}

impl LayoutContext {
    /// Constructs a new `LayoutContext`.
    pub fn new(context: RequestContext, head: PageHead) -> Self {
        LayoutContext {
            context,
            head,
        }
    }

    /// Adds a `<title>` element to the page head.
    pub fn title(&mut self, title: impl Into<String>) {
        self.head.title(title.into());
    }

    /// Adds a `<meta>` element to the page head.
    pub fn metadata(&mut self, metadata: Metadata) {
        self.head.metadata(metadata);
    }

    /// Adds a `<link>` element to the page head.
    pub fn links(&mut self, links: PageLinks) {
        self.head.links(links);
    }

    /// Adds a `<script>` element to the page body.
    pub fn scripts(&mut self, scripts: PageScripts) {
        self.head.scripts(scripts);
    }
}

impl Deref for LayoutContext {
    type Target = RequestContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
