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
use yew::{html::ChildrenProps, BaseComponent};

#[allow(unused_macros)]
macro_rules! server_only {
    () => {
        panic!("this is server only code");
    };
}

/// Contains information about the current request and allow the render the page to respond.
pub struct RenderContext {
    context: RequestContext,
    head: PageHead,

    #[allow(dead_code)]
    render_layout: RenderLayout,
}

#[cfg(not(target_arch = "wasm32"))]
impl RenderContext {
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
        }
    }
}
impl RenderContext {
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

impl RenderContext {
    /// Returns a `404` error.
    pub fn not_found(self) -> Result<Response, Error> {
        Err(ResponseError::from_status(StatusCode::NOT_FOUND).into())
    }

    /// Render the page and returns the `text/html` response.
    pub async fn render<COMP, C>(self) -> Response
    where
        C: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Default + Serialize + Send + Clone,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::web::{Html, IntoResponse};

            // Return a text/html response
            match self.render_html::<COMP, C>().await {
                Ok(html) => Html(html).into_response(),
                Err(err) => ResponseError::from_error(err).into_response(),
            }
        }
    }

    /// Render the page with the given props and returns the `text/html` response.
    #[allow(unused_variables)]
    pub async fn render_with_props<COMP, C>(self, props: COMP::Properties) -> Response
    where
        C: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Serialize + Send + Clone,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::web::{Html, IntoResponse};

            // Return a text/html response
            match self.render_html_with_props::<COMP, C>(props).await {
                Ok(html) => Html(html).into_response(),
                Err(err) => ResponseError::from_error(err).into_response(),
            }
        }
    }

    /// Render the page and returns the `text/html` response stream.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_stream<COMP, C>(self) -> Response
    where
        C: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Default + Serialize + Send + Clone,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::web::{IntoResponse, StreamResponse};

            // Return a stream text/html response
            match self.render_html_stream::<COMP, C>().await {
                Ok(stream) => StreamResponse(stream).into_response(),
                Err(err) => ResponseError::from_error(err).into_response(),
            }
        }
    }

    /// Render the page with the given props and returns the `text/html` response stream.
    #[allow(unused_variables)]
    pub async fn render_stream_with_props<COMP, C>(self, props: COMP::Properties) -> Response
    where
        C: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Serialize + Send + Clone,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::web::{IntoResponse, StreamResponse};

            // Return a stream text/html response
            match self.render_html_stream_with_props::<COMP, C>(props).await {
                Ok(stream) => StreamResponse(stream).into_response(),
                Err(err) => ResponseError::from_error(err).into_response(),
            }
        }
    }

    /// Renders the given component to html.
    pub async fn render_html<COMP, C>(self) -> Result<String, Error>
    where
        C: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Default + Serialize + Send + Clone,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        self.render_html_with_props::<COMP, C>(COMP::Properties::default())
            .await
    }

    /// Renders the given component with the specified props to html.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_html_with_props<COMP, C>(
        self,
        props: COMP::Properties,
    ) -> Result<String, Error>
    where
        C: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Serialize + Send + Clone,
    {
        use crate::{
            app::LayoutContext,
            server::{render_page_to_html, render_to_static_html, RenderPageOptions},
        };

        let request_context = self.context;
        let head = self.head;
        let router = request_context.inner.client_router.clone();
        let error_router = request_context.inner.error_router.clone();
        let error = request_context.inner.error.clone();

        // Render the html template where the content will be rendered
        let index_html = render_to_static_html({
            let render_layout = self.render_layout;
            let request_context = request_context.clone();
            let head = head.clone();
            let layout_ctx = LayoutContext::new(request_context, head);

            move || {
                // We need to block to pass the node to the function because the returned type is no `Send`
                let layout_node = futures::executor::block_on(render_layout(layout_ctx));
                layout_node
            }
        })
        .await;

        let options = RenderPageOptions {
            head,
            error,
            index_html,
            router,
            error_router,
            request_context,
        };

        let result_html = render_page_to_html::<COMP, C>(props, options).await?;
        Ok(result_html)
    }

    /// Renders the given component with the specified props to html.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_html_with_props<COMP, C>(self, _: COMP::Properties) -> Result<String, Error>
    where
        C: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Serialize + Send + Clone,
    {
        server_only!();
    }

    /// Renders the given component to html.
    pub async fn render_html_stream<COMP, C>(
        self,
    ) -> Result<crate::types::TryBoxStream<bytes::Bytes>, Error>
    where
        C: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Default + Serialize + Send + Clone,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        self.render_html_stream_with_props::<COMP, C>(COMP::Properties::default())
            .await
    }

    /// Renders the given component with the specified props to html.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_html_stream_with_props<COMP, C>(
        self,
        props: COMP::Properties,
    ) -> Result<crate::types::TryBoxStream<bytes::Bytes>, Error>
    where
        C: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Serialize + Send + Clone,
    {
        use crate::{
            app::LayoutContext,
            server::{render_page_to_stream, render_to_static_html, RenderPageOptions},
        };

        let request_context = self.context;
        let head = self.head;
        let router = request_context.inner.client_router.clone();
        let error_router = request_context.inner.error_router.clone();
        let error = request_context.inner.error.clone();

        // Render the html template where the content will be rendered
        let index_html = render_to_static_html({
            let render_layout = self.render_layout;
            let request_context = request_context.clone();
            let head = head.clone();
            let layout_ctx = LayoutContext::new(request_context, head);

            move || {
                // We need to block to pass the node to the function because the returned type is no `Send`
                let layout_node = futures::executor::block_on(render_layout(layout_ctx));
                layout_node
            }
        })
        .await;

        let options = RenderPageOptions {
            head,
            error,
            index_html,
            router,
            error_router,
            request_context,
        };

        let result_html = render_page_to_stream::<COMP, C>(props, options).await?;
        Ok(result_html)
    }

    /// Renders the given component with the specified props to html.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_html_stream_with_props<COMP, C>(
        self,
        _: COMP::Properties,
    ) -> Result<crate::types::TryBoxStream<bytes::Bytes>, Error>
    where
        C: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Serialize + Send + Clone,
    {
        server_only!();
    }
}
