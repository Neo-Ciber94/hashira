use super::{layout_data::PageLayoutData, RequestContext};
use crate::server::{Metadata, PageLinks, PageScripts};
use std::ops::Deref;

pub struct LayoutContext {
    context: RequestContext,
    layout_data: PageLayoutData,
}

impl LayoutContext {
    pub fn new(context: RequestContext, layout_data: PageLayoutData) -> Self {
        LayoutContext {
            context,
            layout_data,
        }
    }

    /// Adds a `<meta>` element to the page head.
    pub fn add_metadata(&mut self, metadata: Metadata) {
        self.layout_data.add_metadata(metadata);
    }

    /// Adds a `<link>` element to the page head.
    pub fn add_links(&mut self, links: PageLinks) {
        self.layout_data.add_links(links);
    }

    /// Adds a `<script>` element to the page body.
    pub fn add_scripts(&mut self, scripts: PageScripts) {
        self.layout_data.add_scripts(scripts);
    }
}

impl Deref for LayoutContext {
    type Target = RequestContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
