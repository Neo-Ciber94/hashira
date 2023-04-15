use super::{error_router::ErrorRouter, router::PageRouterWrapper, RenderLayout, Params};
pub use crate::error::ResponseError;
use crate::web::Request;
use std::sync::Arc;

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
struct RequestContextInner {
    path: String,
    params: Params,
    client_router: PageRouterWrapper,
    error_router: Arc<ErrorRouter>,
    request: Arc<Request>,
    error: Option<ResponseError>,

    // TODO: The request context should no had access to the `RenderLayout`,
    // but currently this is the way we get the layout to the `RenderContext`,
    // we should find other way around that.
    //
    // What is preventing to move this from here is: App::page,
    // in that method we should create a RenderContext which require the RequestContext,
    // from that render context is where the actual component is being rendered,
    // where the call ends here
    render_layout: RenderLayout,
}

// FIXME: We could let this type to be cloneable after we move the `render_layout` from here

/// Contains information about the current request.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub struct RequestContext {
    inner: Arc<RequestContextInner>,
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
impl RequestContext {
    pub fn new(
        request: Arc<Request>,
        client_router: PageRouterWrapper,
        error_router: Arc<ErrorRouter>,
        error: Option<ResponseError>,
        render_layout: RenderLayout,
        path: String,
        params: Params,
    ) -> Self {
        let inner = RequestContextInner {
            path,
            params,
            error,
            request,
            client_router,
            error_router,
            render_layout,
        };

        RequestContext {
            inner: Arc::new(inner),
        }
    }
}

impl RequestContext {
    /// Returns the path of the current request.
    pub fn path(&self) -> &str {
        self.inner.path.as_str()
    }

    /// Returns the current request.
    pub fn request(&self) -> &Request {
        self.inner.request.as_ref()
    }

    /// Returns the matching params of the route.
    pub fn params(&self) -> &Params {
        &self.inner.params
    }

    /// Renders the given component to html.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render<COMP, C>(self, head: super::page_head::PageHead) -> String
    where
        COMP: crate::components::PageComponent,
        COMP::Properties: serde::Serialize + Default + Send + Clone,
        C: yew::BaseComponent<Properties = yew::html::ChildrenProps>,
    {
        let props = COMP::Properties::default();
        self.render_with_props::<COMP, C>(props, head).await
    }

    /// Renders the given component with the specified props to html.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_with_props<COMP, C>(
        self,
        props: COMP::Properties,
        head: super::page_head::PageHead,
    ) -> String
    where
        COMP: crate::components::PageComponent,
        COMP::Properties: serde::Serialize + Send + Clone,
        C: yew::BaseComponent<Properties = yew::html::ChildrenProps>,
    {
        use crate::{
            app::LayoutContext,
            server::{render_page_to_html, render_to_static_html, RenderPageOptions},
        };

        let client_router = self.inner.client_router.clone();
        let error_router = self.inner.error_router.clone();
        let error = self.inner.error.clone();
        let render_layout = self.inner.render_layout.clone();

        //
        let request_context = clone_request_context(&self);
        let layout_ctx = LayoutContext::new(self, head.clone());
        let layout_node = render_layout(layout_ctx).await;
        let index_html = render_to_static_html(move || layout_node).await;

        let options = RenderPageOptions {
            head,
            error,
            index_html,
            router: client_router,
            error_router,
            request_context,
        };

        let result_html = render_page_to_html::<COMP, C>(props, options)
            .await
            .unwrap();
        result_html
    }
}

// Required to use the `RequestContext` in a context
impl PartialEq for RequestContext {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

// A helper method to prevent cloning the context directly
pub(crate) fn clone_request_context(ctx: &RequestContext) -> RequestContext {
    RequestContext {
        inner: ctx.inner.clone(),
    }
}
