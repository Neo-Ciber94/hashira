use crate::server::{Metadata, PageLinks, PageScripts};
use std::sync::{Arc, Mutex};

pub(crate) struct Inner {
    // The <title> of the page.
    pub(crate) title: Option<String>,

    // The `<meta>` tags of the page to render
    pub(crate) metadata: Metadata,

    // the <link> tags of the page to render
    pub(crate) links: PageLinks,

    // the <script> tags of the page to render
    pub(crate) scripts: PageScripts,
}

pub struct PageHead(Arc<Mutex<Inner>>); // 

impl PageHead {
    pub(crate) fn new() -> Self {
        let inner = Arc::new(Mutex::new(Inner {
            title: None,
            metadata: Default::default(),
            links: Default::default(),
            scripts: Default::default(),
        }));

        PageHead(inner)
    }

    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(crate) fn into_parts(self) -> (Option<String>, Metadata, PageLinks, PageScripts) {
        let mut inner = self.0.lock().unwrap();
        let title = inner.title.take();
        let metadata = inner.metadata.clone();
        let links = inner.links.clone();
        let scripts = inner.scripts.clone();
        (title, metadata, links, scripts)
    }

    /// Adds a `<title>` element to the page head.
    pub fn title(&mut self, title: impl Into<String>) {
        self.0.lock().unwrap().title.replace(title.into());
    }

    /// Adds a `<meta>` element to the page head.
    pub fn metadata(&mut self, metadata: Metadata) {
        self.0.lock().unwrap().metadata.extend(metadata);
    }

    /// Adds a `<link>` element to the page head.
    pub fn links(&mut self, links: PageLinks) {
        self.0.lock().unwrap().links.extend(links);
    }

    /// Adds a `<script>` element to the page body.
    pub fn scripts(&mut self, scripts: PageScripts) {
        self.0.lock().unwrap().scripts.extend(scripts);
    }
}

impl Clone for PageHead {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Default for PageHead {
    fn default() -> Self {
        PageHead::new()
    }
}
