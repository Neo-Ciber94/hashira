use super::{layout_data::PageLayoutData, RequestContext};
pub use crate::error::ResponseError;
use crate::{error::Error, web::Request};
use crate::{
    server::{Metadata, PageLinks, PageScripts},
    web::Response,
};
use http::StatusCode;
use route_recognizer::Params;
use serde::Serialize;
use std::marker::PhantomData;
use yew::{html::ChildrenProps, BaseComponent};

/// Contains information about the current request and allow the render the page to respond.
pub struct RenderContext<COMP, C> {
    context: RequestContext,
    layout_data: PageLayoutData,
    _marker: PhantomData<(COMP, C)>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<COMP, C> RenderContext<COMP, C> {
    /// Constructs a new render context from the given request context.
    pub(crate) fn new(context: RequestContext, layout_data: PageLayoutData) -> Self {
        RenderContext {
            context,
            layout_data,
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
    pub fn add_title(&mut self, title: impl Into<String>) {
        self.layout_data.add_title(title);
    }

    /// Adds a `<meta>` element to the page head.
    pub fn add_metadata(&mut self, metadata: Metadata) {
        self.layout_data.add_metadata(metadata);
    }

    /// Adds a `<link>` element to the page head.
    pub fn add_links(&mut self, links: PageLinks) {
        self.layout_data.add_links(links);
    }

    /// Adds a `<script>` element to the page body.
    pub fn add_scripts(&mut self, scripts: PageScripts) {
        self.layout_data.add_scripts(scripts);
    }
}

impl<COMP, C> RenderContext<COMP, C>
where
    C: BaseComponent<Properties = ChildrenProps>,
    COMP: BaseComponent,
    COMP::Properties: Serialize + Send + Clone,
{
    /// Render the page and returns the `text/html` response.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render(self) -> Response
    where
        COMP::Properties: Default,
    {
        use crate::web::ResponseExt;

        let layout_data = self.layout_data;
        let html = self.context.render::<COMP, C>(layout_data).await;

        Response::html(html)
    }

    /// Render the page with the given props and returns the `text/html` response.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render_with_props(self, props: COMP::Properties) -> Response {
        use crate::web::ResponseExt;

        let layout_data = self.layout_data;
        let html = self
            .context
            .render_with_props::<COMP, C>(props, layout_data)
            .await;

        Response::html(html)
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

    /// Returns a `404` error.
    pub fn not_found(self) -> Result<Response, Error> {
        Err(ResponseError::from_status(StatusCode::NOT_FOUND).into())
    }
}
