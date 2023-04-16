use super::RenderLayout;
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

#[allow(unused_macros)]
macro_rules! server_only {
    () => {
        panic!("this is server only code");
    };
}

/// Contains information about the current request and allow the render the page to respond.
pub struct RenderContext<COMP, C> {
    context: RequestContext,
    head: PageHead,
    _marker: PhantomData<(COMP, C)>,

    #[allow(dead_code)]
    render_layout: RenderLayout,
}

#[cfg(not(target_arch = "wasm32"))]
impl<COMP, C> RenderContext<COMP, C> {
    /// Constructs a new render context from the given request context.
    pub(crate) fn new(
        context: RequestContext,
        head: PageHead,
        render_layout: RenderLayout,
    ) -> Self {
        RenderContext {
            render_layout,
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
    /// Returns a `404` error.
    pub fn not_found(self) -> Result<Response, Error> {
        Err(ResponseError::from_status(StatusCode::NOT_FOUND).into())
    }

    /// Render the page and returns the `text/html` response.
    pub async fn render(self) -> Response
    where
        COMP::Properties: Default,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::web::{Html, IntoResponse};

            // Return a text/html response
            match self.render_html().await {
                Ok(html) => Html(html).into_response(),
                Err(err) => ResponseError::from_error(err).into_response(),
            }
        }
    }

    /// Render the page with the given props and returns the `text/html` response.
    #[allow(unused_variables)]
    pub async fn render_with_props(self, props: COMP::Properties) -> Response {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::web::{Html, IntoResponse};

            // Return a text/html response
            match self.render_html_with_props(props).await {
                Ok(html) => Html(html).into_response(),
                Err(err) => ResponseError::from_error(err).into_response(),
            }
        }
    }

    /// Render the page and returns the `text/html` response stream.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_stream(self) -> Response
    where
        COMP::Properties: Default,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::web::{IntoResponse, StreamResponse};

            // Return a stream text/html response
            match self.render_html_stream().await {
                Ok(stream) => StreamResponse(stream).into_response(),
                Err(err) => ResponseError::from_error(err).into_response(),
            }
        }
    }

    /// Render the page with the given props and returns the `text/html` response stream.
    #[allow(unused_variables)]
    pub async fn render_stream_with_props(self, props: COMP::Properties) -> Response {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::web::{IntoResponse, StreamResponse};

            // Return a stream text/html response
            match self.render_html_stream_with_props(props).await {
                Ok(stream) => StreamResponse(stream).into_response(),
                Err(err) => ResponseError::from_error(err).into_response(),
            }
        }
    }

    /// Renders the given component to html.
    pub async fn render_html(self) -> Result<String, Error>
    where
        COMP: crate::components::PageComponent,
        COMP::Properties: serde::Serialize + Default + Send + Clone,
        C: yew::BaseComponent<Properties = yew::html::ChildrenProps>,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        self.render_html_with_props(COMP::Properties::default())
            .await
    }

    /// Renders the given component with the specified props to html.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_html_with_props(self, props: COMP::Properties) -> Result<String, Error>
    where
        COMP: crate::components::PageComponent,
        COMP::Properties: serde::Serialize + Send + Clone,
        C: yew::BaseComponent<Properties = yew::html::ChildrenProps>,
    {
        use crate::{
            app::LayoutContext,
            server::{render_page_to_html, render_to_static_html, RenderPageOptions},
        };

        let req_context = self.context;
        let head = self.head;
        let client_router = req_context.inner.client_router.clone();
        let error_router = req_context.inner.error_router.clone();
        let error = req_context.inner.error.clone();
        let render_layout = self.render_layout;

        //
        let request_context = req_context.clone();
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

    /// Renders the given component with the specified props to html.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_html_with_props(self, _: COMP::Properties) -> Result<String, Error>
    where
        COMP: crate::components::PageComponent,
        COMP::Properties: serde::Serialize + Send + Clone,
        C: yew::BaseComponent<Properties = yew::html::ChildrenProps>,
    {
        server_only!();
    }

    /// Renders the given component to html.
    pub async fn render_html_stream(self) -> Result<crate::types::TryBoxStream<bytes::Bytes>, Error>
    where
        COMP: crate::components::PageComponent,
        COMP::Properties: serde::Serialize + Default + Send + Clone,
        C: yew::BaseComponent<Properties = yew::html::ChildrenProps>,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        self.render_html_stream_with_props(COMP::Properties::default())
            .await
    }

    /// Renders the given component with the specified props to html.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_html_stream_with_props(
        self,
        props: COMP::Properties,
    ) -> Result<crate::types::TryBoxStream<bytes::Bytes>, Error>
    where
        COMP: crate::components::PageComponent,
        COMP::Properties: serde::Serialize + Send + Clone,
        C: yew::BaseComponent<Properties = yew::html::ChildrenProps>,
    {
        use crate::{
            app::LayoutContext,
            server::{render_page_to_stream, render_to_static_html, RenderPageOptions},
        };

        let req_context = self.context;
        let head = self.head;
        let client_router = req_context.inner.client_router.clone();
        let error_router = req_context.inner.error_router.clone();
        let error = req_context.inner.error.clone();
        let render_layout = self.render_layout;
        //
        let request_context = req_context.clone();
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

        let result_html = render_page_to_stream::<COMP, C>(props, options).await?;
        Ok(result_html)
    }

    /// Renders the given component with the specified props to html.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_html_stream_with_props(
        self,
        _: COMP::Properties,
    ) -> Result<crate::types::TryBoxStream<bytes::Bytes>, Error>
    where
        COMP: crate::components::PageComponent,
        COMP::Properties: serde::Serialize + Send + Clone,
        C: yew::BaseComponent<Properties = yew::html::ChildrenProps>,
    {
        server_only!();
    }
}
