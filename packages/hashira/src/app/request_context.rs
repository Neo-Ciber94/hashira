use super::{client_router::ClientRouter, error_router::ClientErrorRouter, RenderLayout};
pub use crate::error::ResponseError;
use crate::{
    server::{Metadata, PageLinks, PageScripts},
    web::Request,
};
use route_recognizer::Params;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use yew::{html::ChildrenProps, BaseComponent};

struct PageLayoutData {
    // The `<meta>` tags of the page to render
    metadata: Metadata,

    // the <link> tags of the page to render
    links: PageLinks,

    // the <script> tags of the page to render
    scripts: PageScripts,
}

/// Contains information about the current request.
#[allow(dead_code)] // TODO: Ignore server only data
pub struct RequestContext<C> {
    path: String,
    params: Params,
    client_router: ClientRouter,
    client_error_router: Arc<ClientErrorRouter>,
    request: Option<Arc<Request>>,
    error: Option<ResponseError>,
    layout: Option<RenderLayout<C>>,
    data: Arc<Mutex<PageLayoutData>>,
}

#[allow(dead_code)] // TODO: Ignore server only data
impl<C> RequestContext<C> {
    pub fn new(
        request: Option<Arc<Request>>,
        client_router: ClientRouter,
        client_error_router: Arc<ClientErrorRouter>,
        error: Option<ResponseError>,
        path: String,
        layout: RenderLayout<C>,
        params: Params,
    ) -> Self {
        let data = PageLayoutData {
            metadata: Metadata::default(),
            links: PageLinks::default(),
            scripts: PageScripts::default(),
        };

        RequestContext {
            path,
            params,
            error,
            request,
            client_router,
            layout: Some(layout),
            client_error_router,
            data: Arc::new(Mutex::new(data)),
        }
    }
}

impl<C> RequestContext<C>
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    /// Adds a `<meta>` element to the page head.
    pub fn add_metadata(&mut self, metadata: Metadata) {
        self.data.lock().unwrap().metadata.extend(metadata);
    }

    /// Adds a `<link>` element to the page head.
    pub fn add_links(&mut self, links: PageLinks) {
        self.data.lock().unwrap().links.extend(links);
    }

    /// Adds a `<script>` element to the page body.
    pub fn add_scripts(&mut self, scripts: PageScripts) {
        self.data.lock().unwrap().scripts.extend(scripts);
    }

    /// Returns the current request.
    pub fn request(&self) -> &Request {
        self.request
            .as_ref()
            .expect("no request is being processed")
    }

    /// Returns the matching params of the route.
    pub fn params(&self) -> &Params {
        &self.params
    }

    /// Renders the given component to html.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render<COMP>(self) -> String
    where
        COMP: BaseComponent,
        COMP::Properties: Serialize + Default + Send + Clone,
    {
        let props = COMP::Properties::default();
        self.render_with_props::<COMP>(props).await
    }

    /// Renders the given component with the specified props to html.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_with_props<COMP>(self, props: COMP::Properties) -> String
    where
        COMP: BaseComponent,
        COMP::Properties: Serialize + Send + Clone,
    {
        use crate::server::{render_page_to_html, render_to_static_html, RenderPageOptions};

        let Self {
            layout,
            request,
            data: inner,
            params,
            client_router,
            client_error_router,
            path,
            error,
        } = self;

        let render_layout = layout.unwrap();

        let layout_ctx = RequestContext {
            params,
            request,
            path: path.clone(),
            layout: None,
            error: None, // FIXME: Pass error to layout?
            client_router: client_router.clone(),
            client_error_router: client_error_router.clone(),
            data: inner.clone(),
        };

        let layout_node = render_layout(layout_ctx).await;
        let layout = render_to_static_html(move || layout_node).await;

        let inner = inner.lock().unwrap();
        let links = inner.links.clone();
        let metadata = inner.metadata.clone();
        let scripts = inner.scripts.clone();

        let options = RenderPageOptions {
            path,
            error,
            layout,
            metadata,
            links,
            scripts,
            client_router,
            client_error_router,
        };

        let result_html = render_page_to_html::<COMP, C>(props, options)
            .await
            .unwrap();
        result_html
    }
}
