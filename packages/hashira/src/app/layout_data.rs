use crate::server::{Metadata, PageLinks, PageScripts};
use std::sync::{Arc, Mutex};

pub(crate) struct Inner {
    // The `<meta>` tags of the page to render
    pub(crate) metadata: Metadata,

    // the <link> tags of the page to render
    pub(crate) links: PageLinks,

    // the <script> tags of the page to render
    pub(crate) scripts: PageScripts,
}

pub struct PageLayoutData(Arc<Mutex<Inner>>);

impl PageLayoutData {
    pub(crate) fn new() -> Self {
        let inner = Arc::new(Mutex::new(Inner {
            metadata: Default::default(),
            links: Default::default(),
            scripts: Default::default(),
        }));

        PageLayoutData(inner)
    }

    pub(crate) fn into_parts(self) -> (Metadata, PageLinks, PageScripts) {
        let inner = self.0.lock().unwrap();
        let metadata = inner.metadata.clone();
        let links = inner.links.clone();
        let scripts = inner.scripts.clone();
        (metadata, links, scripts)
    }

    /// Adds a `<meta>` element to the page head.
    pub fn add_metadata(&mut self, metadata: Metadata) {
        self.0.lock().unwrap().metadata.extend(metadata);
    }

    /// Adds a `<link>` element to the page head.
    pub fn add_links(&mut self, links: PageLinks) {
        self.0.lock().unwrap().links.extend(links);
    }

    /// Adds a `<script>` element to the page body.
    pub fn add_scripts(&mut self, scripts: PageScripts) {
        self.0.lock().unwrap().scripts.extend(scripts);
    }
}

impl Clone for PageLayoutData {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Default for PageLayoutData {
    fn default() -> Self {
        PageLayoutData::new()
    }
}
