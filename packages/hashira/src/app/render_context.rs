use std::ops::Deref;

use super::RenderLayout;
use super::{page_head::PageHead, RequestContext};
use crate::components::PageComponent;
use crate::error::{BoxError, ServerError};
use crate::web::{IntoResponse, Redirect};
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

impl RenderContext {
    /// Constructs a new render context from the given request context.
    #[allow(dead_code)]
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
    pub fn not_found(self) -> Result<Response, BoxError> {
        Err(ServerError::from_status(StatusCode::NOT_FOUND).into())
    }

    /// Returns a redirection.
    ///
    /// # Panic
    /// - If the status is not a valid redirection
    /// - the `to` is no a valid uri
    pub fn redirect(self, to: &str, status: StatusCode) -> Result<Response, BoxError> {
        assert!(
            status.is_redirection(),
            "invalid redirection status code: {status}"
        );

        Ok(Redirect::new(to, status)
            .expect("redirection error")
            .into_response())
    }

    /// Render the page and returns the `text/html` response.
    pub async fn render<COMP, BASE>(self) -> Response
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Default + Serialize + Send,
    {
        use crate::web::Html;

        // Return a text/html response
        match self.render_html::<COMP, BASE>().await {
            Ok(html) => Html(html).into_response(),
            Err(err) => ServerError::from_error(err).into_response(),
        }
    }

    /// Render the page with the given props and returns the `text/html` response.
    #[allow(unused_variables)]
    pub async fn render_with_props<COMP, BASE>(self, props: COMP::Properties) -> Response
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Serialize + Send,
    {
        use crate::web::Html;

        // Return a text/html response
        match self.render_html_with_props::<COMP, BASE>(props).await {
            Ok(html) => Html(html).into_response(),
            Err(err) => ServerError::from_error(err).into_response(),
        }
    }

    /// Render the page and returns the `text/html` response stream.
    pub async fn render_stream<COMP, BASE>(self) -> Response
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Default + Serialize + Send,
    {
        #[cfg(feature = "client")]
        server_only!();

        #[cfg(not(feature = "client"))]
        {
            use crate::web::StreamResponse;

            // Return a stream text/html response
            match self.render_html_stream::<COMP, BASE>().await {
                Ok(stream) => StreamResponse(stream).into_response(),
                Err(err) => ServerError::from_error(err).into_response(),
            }
        }
    }

    /// Render the page with the given props and returns the `text/html` response stream.
    #[allow(unused_variables)]
    pub async fn render_stream_with_props<COMP, BASE>(self, props: COMP::Properties) -> Response
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Serialize + Send,
    {
        #[cfg(feature = "client")]
        server_only!();

        #[cfg(not(feature = "client"))]
        {
            use crate::web::StreamResponse;

            // Return a stream text/html response
            match self
                .render_html_stream_with_props::<COMP, BASE>(props)
                .await
            {
                Ok(stream) => StreamResponse(stream).into_response(),
                Err(err) => ServerError::from_error(err).into_response(),
            }
        }
    }

    /// Renders the given component to html.
    pub async fn render_html<COMP, BASE>(self) -> Result<String, BoxError>
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Default + Serialize + Send,
    {
        #[cfg(feature = "client")]
        server_only!();

        #[cfg(not(feature = "client"))]
        self.render_html_with_props::<COMP, BASE>(COMP::Properties::default())
            .await
    }

    /// Renders the given component with the specified props to html.
    #[cfg_attr(feature = "client", allow(unused_variables))]
    pub async fn render_html_with_props<COMP, BASE>(
        self,
        props: COMP::Properties,
    ) -> Result<String, BoxError>
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Serialize + Send,
    {
        #[cfg(feature = "client")]
        server_only!();

        #[cfg(not(feature = "client"))]
        {
            use crate::server::render_page_to_html;

            let options = self.get_render_options().await;
            let result_html = render_page_to_html::<COMP, BASE>(props, options).await?;
            Ok(result_html)
        }
    }

    /// Renders the given component to html.
    pub async fn render_html_stream<COMP, BASE>(
        self,
    ) -> Result<crate::types::TryBoxStream<bytes::Bytes>, BoxError>
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Default + Serialize + Send,
    {
        #[cfg(feature = "client")]
        server_only!();

        #[cfg(not(feature = "client"))]
        self.render_html_stream_with_props::<COMP, BASE>(COMP::Properties::default())
            .await
    }

    /// Renders the given component with the specified props to html.
    #[cfg_attr(feature = "client", allow(unused_variables))]
    pub async fn render_html_stream_with_props<COMP, BASE>(
        self,
        props: COMP::Properties,
    ) -> Result<crate::types::TryBoxStream<bytes::Bytes>, BoxError>
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Serialize + Send,
    {
        #[cfg(feature = "client")]
        server_only!();

        #[cfg(not(feature = "client"))]
        {
            use crate::server::render_page_to_stream;

            let options = self.get_render_options().await;
            let result_html = render_page_to_stream::<COMP, BASE>(props, options).await?;
            Ok(result_html)
        }
    }

    #[cfg(not(feature = "client"))]
    async fn get_render_options(&self) -> crate::server::RenderPageOptions {
        use crate::{
            app::LayoutContext,
            server::{render_to_static_html, RenderPageOptions},
        };

        let request_context = self.context.clone();
        let head = self.head.clone();
        let router = request_context.inner.client_router.clone();
        let error_router = request_context.inner.error_router.clone();

        let layout_head = PageHead::new();
        let index_html: String;

        // Render the html template where the content will be rendered
        {
            #[cfg(not(target_arch = "wasm32"))]
            {
                index_html = render_to_static_html({
                    let render_layout = self.render_layout.clone();
                    let request_context = request_context.clone();
                    let head = layout_head.clone();
                    let layout_ctx = LayoutContext::new(request_context, head);

                    move || {
                        // We need to block to pass the node to the function because the returned type is no `Send`
                        futures::executor::block_on(render_layout(layout_ctx))
                    }
                })
                .await;
            }

            // On wasm targets we cannot block the thread,
            // we use `Fragile` to safely send the `VNode` which is no `Send` to the render function,
            // this is safe because wasm is single threaded
            #[cfg(target_arch = "wasm32")]
            {
                index_html = render_to_static_html({
                    let render_layout = self.render_layout.clone();
                    let request_context = request_context.clone();
                    let head = layout_head.clone();
                    let layout_ctx = LayoutContext::new(request_context, head);

                    //let node = render_layout(layout_ctx).await;
                    let node = render_layout(layout_ctx).await;
                    let fragile = fragile::Fragile::new(node);

                    move || {
                        // SAFETY: This is safe because wasm only run one thread
                        fragile.into_inner()
                    }
                })
                .await;
            }
        }

        // Merge the layout head with the current component head
        let head = layout_head.merge(head);

        RenderPageOptions {
            head,
            index_html,
            router,
            error_router,
            request_context,
        }
    }
}

impl Deref for RenderContext {
    type Target = RequestContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
