use super::{page_head::PageHead, RequestContext};
use crate::components::PageComponent;
pub use crate::error::ResponseError;
use crate::routing::Params;
use crate::{error::Error, web::Request};
use crate::{
    server::{Metadata, PageLinks, PageScripts},
    web::Response,
};
use http::StatusCode;
use serde::Serialize;
use std::marker::PhantomData;
use yew::{html::ChildrenProps, BaseComponent};

/// Contains information about the current request and allow the render the page to respond.
pub struct RenderContext<COMP, C> {
    context: RequestContext,
    head: PageHead,
    _marker: PhantomData<(COMP, C)>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<COMP, C> RenderContext<COMP, C> {
    /// Constructs a new render context from the given request context.
    pub(crate) fn new(context: RequestContext, head: PageHead) -> Self {
        RenderContext {
            context,
            head,
            _marker: PhantomData,
        }
    }
}
impl<COMP, C> RenderContext<COMP, C>
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    /// Returns the path of the current request.
    pub fn path(&self) -> &str {
        self.context.path()
    }

    /// Returns the current request.
    pub fn request(&self) -> &Request {
        self.context.request()
    }

    /// Returns the matching params of the route.
    pub fn params(&self) -> &Params {
        self.context.params()
    }

    /// Adds a `<title>` element to the page head.
    pub fn title(&mut self, title: impl Into<String>) {
        self.head.title(title);
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

impl<COMP, C> RenderContext<COMP, C>
where
    C: BaseComponent<Properties = ChildrenProps>,
    COMP: PageComponent,
    COMP::Properties: Serialize + Send + Clone,
{
    /// Render the page and returns the `text/html` response.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render(self) -> Response
    where
        COMP::Properties: Default,
    {
        use crate::web::{Html, IntoResponse};

        let head = self.head;

        // Return a text/html response
        match self.context.render::<COMP, C>(head).await {
            Ok(html) => Html(html).into_response(),
            Err(err) => ResponseError::from_error(err).into_response(),
        }
    }

    /// Render the page with the given props and returns the `text/html` response.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_with_props(self, props: COMP::Properties) -> Response {
        use crate::web::{Html, IntoResponse};

        let head = self.head;

        // Return a text/html response
        match self.context.render_with_props::<COMP, C>(props, head).await {
            Ok(html) => Html(html).into_response(),
            Err(err) => ResponseError::from_error(err).into_response(),
        }
    }

    /// Render the page and returns the `text/html` response stream.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_stream(self) -> Response
    where
        COMP::Properties: Default,
    {
        use crate::web::{IntoResponse, StreamResponse};

        let head = self.head;

        // Return a stream text/html response
        match self.context.render_stream::<COMP, C>(head).await {
            Ok(stream) => StreamResponse(stream).into_response(),
            Err(err) => ResponseError::from_error(err).into_response(),
        }
    }

    /// Render the page with the given props and returns the `text/html` response stream.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_stream_with_props(self, props: COMP::Properties) -> Response {
        use crate::web::{IntoResponse, StreamResponse};

        let head = self.head;

        // Return a stream text/html response
        match self
            .context
            .render_stream_with_props::<COMP, C>(props, head)
            .await
        {
            Ok(stream) => StreamResponse(stream).into_response(),
            Err(err) => ResponseError::from_error(err).into_response(),
        }
    }

    /// Render the page and returns the `text/html` response.
    #[cfg(target_arch = "wasm32")]
    pub async fn render(self) -> Response
    where
        COMP::Properties: Default,
    {
        unreachable!("this is a server-only function")
    }

    /// Render the page with the given props and returns the `text/html` response.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_with_props(self, _: COMP::Properties) -> Response {
        unreachable!("this is a server-only function")
    }

    /// Render the page and returns the `text/html` response stream.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_stream(self) -> Response
    where
        COMP::Properties: Default,
    {
        unreachable!("this is a server-only function")
    }

    /// Render the page with the given props and returns the `text/html` response stream.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_stream_with_props(self, _: COMP::Properties) -> Response {
        unreachable!("this is a server-only function")
    }

    /// Returns a `404` error.
    pub fn not_found(self) -> Result<Response, Error> {
        Err(ResponseError::from_status(StatusCode::NOT_FOUND).into())
    }

    /// Renders the given component to html.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_html(self, head: super::page_head::PageHead) -> Result<String, Error>
    where
        COMP: crate::components::PageComponent,
        COMP::Properties: serde::Serialize + Default + Send + Clone,
        C: yew::BaseComponent<Properties = yew::html::ChildrenProps>,
    {
        let props = COMP::Properties::default();
        self.render_html_with_props(props, head).await
    }

    /// Renders the given component with the specified props to html.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_html_with_props(
        self,
        props: COMP::Properties,
        head: super::page_head::PageHead,
    ) -> Result<String, Error>
    where
        COMP: crate::components::PageComponent,
        COMP::Properties: serde::Serialize + Send + Clone,
        C: yew::BaseComponent<Properties = yew::html::ChildrenProps>,
    {
        use crate::{
            app::{LayoutContext, clone_request_context},
            server::{render_page_to_html, render_to_static_html, RenderPageOptions},
        };

        let req_context = self.context;
        let client_router = req_context.inner.client_router.clone();
        let error_router = req_context.inner.error_router.clone();
        let error = req_context.inner.error.clone();
        let render_layout = req_context.inner.render_layout.clone();

        //
        let request_context = clone_request_context(&req_context);
        let layout_ctx = LayoutContext::new(req_context, head.clone());
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

        let result_html = render_page_to_html::<COMP, C>(props, options).await?;
        Ok(result_html)
    }
}
