use super::layout_data::PageLayoutData;
use super::{error_router::ErrorRouter, router::ClientRouter, RenderLayout};
pub use crate::error::ResponseError;
use crate::web::Request;
use route_recognizer::Params;
use std::sync::Arc;
use yew::{html::ChildrenProps, BaseComponent};

/// Contains information about the current request.
#[allow(dead_code)] // TODO: Ignore server only data
pub struct RequestContext {
    path: String,
    params: Params,
    client_router: ClientRouter,
    error_router: Arc<ErrorRouter>,
    request: Option<Arc<Request>>,
    error: Option<ResponseError>,
}

#[allow(dead_code)] // TODO: Ignore server only data
impl RequestContext {
    pub fn new(
        request: Option<Arc<Request>>,
        client_router: ClientRouter,
        error_router: Arc<ErrorRouter>,
        error: Option<ResponseError>,
        path: String,
        params: Params,
    ) -> Self {
        RequestContext {
            path,
            params,
            error,
            request,
            client_router,
            error_router,
        }
    }
}

impl RequestContext {
    /// Returns the path of the current request.
    pub fn path(&self) -> &str {
        self.path.as_str()
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
    pub async fn render<COMP, C>(
        self,
        layout_data: PageLayoutData,
        render_layout: RenderLayout,
    ) -> String
    where
        COMP: BaseComponent,
        COMP::Properties: serde::Serialize + Default + Send + Clone,
        C: BaseComponent<Properties = ChildrenProps>,
    {
        let props = COMP::Properties::default();
        self.render_with_props::<COMP, C>(props, layout_data, render_layout)
            .await
    }

    /// Renders the given component with the specified props to html.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_with_props<COMP, C>(
        self,
        props: COMP::Properties,
        layout_data: PageLayoutData,
        render_layout: RenderLayout,
    ) -> String
    where
        COMP: BaseComponent,
        COMP::Properties: serde::Serialize + Send + Clone,
        C: BaseComponent<Properties = ChildrenProps>,
    {
        use crate::{
            app::LayoutContext,
            server::{render_page_to_html, RenderPageOptions},
        };

        let Self {
            client_router,
            error_router,
            path,
            error,
            request: _,
            params: _,
        } = self;

        let layout_ctx = LayoutContext::new(layout_data.clone());

        // Call the layout to ensure it sets the metadata
        let _ = render_layout(layout_ctx).await;

        // Reads the index.html from the file system
        let index_html = get_index_html();

        // Get page title, links, meta and scripts
        let (title, metadata, links, scripts) = layout_data.into_parts();

        let options = RenderPageOptions {
            title,
            path,
            error,
            index_html,
            metadata,
            links,
            scripts,
            client_router,
            error_router,
        };

        let result_html = render_page_to_html::<COMP, C>(props, options)
            .await
            .unwrap();
        result_html
    }
}

fn get_index_html() -> String {
    use once_cell::sync::OnceCell;

    static INDEX_HTML: OnceCell<String> = OnceCell::new();

    INDEX_HTML
        .get_or_init(|| {
            let mut public_dir = crate::server::env::get_public_dir();

            if !public_dir.exists() {
                panic!("Public directory was not found: {}", public_dir.display());
            }

            public_dir.push("index.html");
            let index_html = std::fs::read_to_string(public_dir)
                .unwrap_or_else(|e| panic!("failed to read index.html: {e}"));
            index_html
        })
        .clone()
}
