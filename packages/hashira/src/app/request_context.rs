use super::{error_router::ErrorRouter, router::ClientRouter, RenderLayout};
pub use crate::error::ResponseError;
use crate::web::Request;
use route_recognizer::Params;
use std::sync::Arc;

/// Contains information about the current request.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub struct RequestContext {
    path: String,
    params: Params,
    client_router: ClientRouter,
    error_router: Arc<ErrorRouter>,
    request: Option<Arc<Request>>,
    error: Option<ResponseError>,

    // TODO: The request context should no had access to the `RenderLayout`,
    // but currently this is the way we get the layout to the `RenderContext`,
    // we should find other way around that
    render_layout: RenderLayout,
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
impl RequestContext {
    pub fn new(
        request: Option<Arc<Request>>,
        client_router: ClientRouter,
        error_router: Arc<ErrorRouter>,
        error: Option<ResponseError>,
        render_layout: RenderLayout,
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
            render_layout,
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
    pub async fn render<COMP, C>(self, layout_data: super::layout_data::PageLayoutData) -> String
    where
        COMP: yew::BaseComponent,
        COMP::Properties: serde::Serialize + Default + Send + Clone,
        C: yew::BaseComponent<Properties = yew::html::ChildrenProps>,
    {
        let props = COMP::Properties::default();
        self.render_with_props::<COMP, C>(props, layout_data).await
    }

    /// Renders the given component with the specified props to html.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_with_props<COMP, C>(
        self,
        props: COMP::Properties,
        layout_data: super::layout_data::PageLayoutData,
    ) -> String
    where
        COMP: yew::BaseComponent,
        COMP::Properties: serde::Serialize + Send + Clone,
        C: yew::BaseComponent<Properties = yew::html::ChildrenProps>,
    {
        use crate::{
            app::LayoutContext,
            server::{render_page_to_html, render_to_static_html, RenderPageOptions},
        };

        let Self {
            client_router,
            error_router,
            path,
            error,
            request,
            params,
            render_layout,
        } = self;

        let layout_request_ctx = RequestContext {
            params,
            request,
            path: path.clone(),
            render_layout: render_layout.clone(),
            error: None, // FIXME: Pass error to layout?
            client_router: client_router.clone(),
            error_router: error_router.clone(),
        };

        let layout_ctx = LayoutContext::new(layout_request_ctx, layout_data.clone());

        let layout_node = render_layout(layout_ctx).await;
        let index_html = render_to_static_html(move || layout_node).await;

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
