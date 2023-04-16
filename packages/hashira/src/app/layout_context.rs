use super::{page_head::PageHead, RequestContext};
use crate::server::{Metadata, PageLinks, PageScripts};
use std::ops::Deref;

/// The context used to render the layout.
pub struct LayoutContext {
    context: RequestContext,
    layout_data: PageHead,
}

impl LayoutContext {
    /// Constructs a new `LayoutContext`.
    pub fn new(context: RequestContext, layout_data: PageHead) -> Self {
        LayoutContext {
            context,
            layout_data,
        }
    }

    /// Adds a `<title>` element to the page head.
    pub fn add_title(&mut self, title: impl Into<String>) {
        self.layout_data.title(title.into());
    }

    /// Adds a `<meta>` element to the page head.
    pub fn add_metadata(&mut self, metadata: Metadata) {
        self.layout_data.metadata(metadata);
    }

    /// Adds a `<link>` element to the page head.
    pub fn add_links(&mut self, links: PageLinks) {
        self.layout_data.links(links);
    }

    /// Adds a `<script>` element to the page body.
    pub fn add_scripts(&mut self, scripts: PageScripts) {
        self.layout_data.scripts(scripts);
    }
}

impl Deref for LayoutContext {
    type Target = RequestContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
