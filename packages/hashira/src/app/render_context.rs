use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use super::RenderLayout;
use super::{page_head::PageHead, RequestContext};
use crate::components::PageComponent;
pub use crate::error::ResponseError;
use crate::routing::Params;
use crate::web::{IntoResponse, Redirect};
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

    /// Returns a redirection.
    /// 
    /// # Panic
    /// - If the status is not a valid redirection
    /// - the `to` is no a valid uri
    pub fn redirect(self, to: &str, status: StatusCode) -> PageResponse<(), ()> {
        assert!(
            status.is_redirection(),
            "invalid redirection status code: {status}"
        );

        let res = Redirect::new(to, status).expect("redirection error");
        PageResponse::new(res)
    }

    /// Render the page and returns the `text/html` response.
    pub async fn render<COMP, BASE>(self) -> PageResponse<COMP, BASE>
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Default + Serialize + Send + Clone,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::web::Html;

            // Return a text/html response
            match self.render_html::<COMP, BASE>().await {
                Ok(html) => PageResponse::new(Html(html)),
                Err(err) => PageResponse::new(ResponseError::from_error(err)),
            }
        }
    }

    /// Render the page with the given props and returns the `text/html` response.
    #[allow(unused_variables)]
    pub async fn render_with_props<COMP, BASE>(self, props: COMP::Properties) -> PageResponse<COMP, BASE>
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Serialize + Send + Clone,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::web::Html;

            // Return a text/html response
            match self.render_html_with_props::<COMP, BASE>(props).await {
                Ok(html) => PageResponse::new(Html(html)),
                Err(err) => PageResponse::new(ResponseError::from_error(err)),
            }
        }
    }

    /// Render the page and returns the `text/html` response stream.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_stream<COMP, BASE>(self) -> PageResponse<COMP, BASE>
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Default + Serialize + Send + Clone,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::web::StreamResponse;

            // Return a stream text/html response
            match self.render_html_stream::<COMP, BASE>().await {
                Ok(stream) => PageResponse::new(StreamResponse(stream)),
                Err(err) => PageResponse::new(ResponseError::from_error(err)),
            }
        }
    }

    /// Render the page with the given props and returns the `text/html` response stream.
    #[allow(unused_variables)]
    pub async fn render_stream_with_props<COMP, BASE>(
        self,
        props: COMP::Properties,
    ) -> PageResponse<COMP, BASE>
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Serialize + Send + Clone,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::web::StreamResponse;

            // Return a stream text/html response
            match self.render_html_stream_with_props::<COMP, BASE>(props).await {
                Ok(stream) => PageResponse::new(StreamResponse(stream)),
                Err(err) => PageResponse::new(ResponseError::from_error(err)),
            }
        }
    }

    /// Renders the given component to html.
    pub async fn render_html<COMP, BASE>(self) -> Result<String, Error>
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Default + Serialize + Send + Clone,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        self.render_html_with_props::<COMP, BASE>(COMP::Properties::default())
            .await
    }

    /// Renders the given component with the specified props to html.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_html_with_props<COMP, BASE>(
        self,
        props: COMP::Properties,
    ) -> Result<String, Error>
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
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
                futures::executor::block_on(render_layout(layout_ctx))
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

        let result_html = render_page_to_html::<COMP, BASE>(props, options).await?;
        Ok(result_html)
    }

    /// Renders the given component with the specified props to html.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_html_with_props<COMP, BASE>(self, _: COMP::Properties) -> Result<String, Error>
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Serialize + Send + Clone,
    {
        server_only!();
    }

    /// Renders the given component to html.
    pub async fn render_html_stream<COMP, BASE>(
        self,
    ) -> Result<crate::types::TryBoxStream<bytes::Bytes>, Error>
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Default + Serialize + Send + Clone,
    {
        #[cfg(target_arch = "wasm32")]
        server_only!();

        #[cfg(not(target_arch = "wasm32"))]
        self.render_html_stream_with_props::<COMP, BASE>(COMP::Properties::default())
            .await
    }

    /// Renders the given component with the specified props to html.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_html_stream_with_props<COMP, BASE>(
        self,
        props: COMP::Properties,
    ) -> Result<crate::types::TryBoxStream<bytes::Bytes>, Error>
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
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
                futures::executor::block_on(render_layout(layout_ctx))
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

        let result_html = render_page_to_stream::<COMP, BASE>(props, options).await?;
        Ok(result_html)
    }

    /// Renders the given component with the specified props to html.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_html_stream_with_props<COMP, BASE>(
        self,
        _: COMP::Properties,
    ) -> Result<crate::types::TryBoxStream<bytes::Bytes>, Error>
    where
    BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Serialize + Send + Clone,
    {
        server_only!();
    }
}

/// Represents the response out a page route.
pub struct PageResponse<COMP, BASE> {
    response: Response,
    _marker: PhantomData<(COMP, BASE)>,
}

impl<COMP, BASE> Deref for PageResponse<COMP, BASE> {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl<COMP, BASE> DerefMut for PageResponse<COMP, BASE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.response
    }
}

impl<COMP, BASE> PageResponse<COMP, BASE> {
    #[allow(dead_code)]
    pub(crate) fn new<T: IntoResponse>(response: T) -> Self {
        let response = response.into_response();

        PageResponse {
            response,
            _marker: PhantomData,
        }
    }
}

impl<COMP, BASE> IntoResponse for PageResponse<COMP, BASE> {
    fn into_response(self) -> Response {
        self.response
    }
}
