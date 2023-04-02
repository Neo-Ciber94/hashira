use super::RenderLayout;
use crate::{
    server::{
        render_page_to_html, render_to_static_html, Metadata, PageLinks, PageScripts,
        RenderPageOptions,
    },
    web::Request,
};
use route_recognizer::Params;
use serde::Serialize;
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};
use yew::{html::ChildrenProps, BaseComponent};

struct AppContextInner {
    // The `<meta>` tags of the page to render
    metadata: Metadata,

    // the <link> tags of the page to render
    links: PageLinks,

    // the <script> tags of the page to render
    scripts: PageScripts,
}

pub struct AppContext<C> {
    request: Option<Request>,
    params: Params,
    layout: Option<RenderLayout<C>>,
    inner: Arc<Mutex<AppContextInner>>,
}

impl<C> AppContext<C> {
    pub fn new(request: Request, layout: RenderLayout<C>, params: Params) -> Self {
        let inner = AppContextInner {
            metadata: Metadata::default(),
            links: PageLinks::default(),
            scripts: PageScripts::default(),
        };

        AppContext {
            params,
            request: Some(request),
            layout: Some(layout),
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub(crate) fn no_request(layout: RenderLayout<C>, params: Params) -> Self {
        let inner = AppContextInner {
            metadata: Metadata::default(),
            links: PageLinks::default(),
            scripts: PageScripts::default(),
        };

        AppContext {
            params,
            request: None,
            layout: Some(layout),
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}

impl<C> AppContext<C>
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    pub fn add_metadata(&mut self, metadata: Metadata) {
        self.inner.lock().unwrap().metadata.extend(metadata);
    }

    pub fn add_links(&mut self, links: PageLinks) {
        self.inner.lock().unwrap().links.extend(links);
    }

    pub fn add_scripts(&mut self, scripts: PageScripts) {
        self.inner.lock().unwrap().scripts.extend(scripts);
    }

    pub fn request(&self) -> &Request {
        self.request
            .as_ref()
            .expect("no request is being processed")
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    pub async fn render<COMP>(self) -> String
    where
        COMP: BaseComponent,
        COMP::Properties: Serialize + Default + Send + Clone,
    {
        let props = COMP::Properties::default();
        self.render_with_props::<COMP>(props).await
    }

    pub async fn render_with_props<COMP>(self, props: COMP::Properties) -> String
    where
        COMP: BaseComponent,
        COMP::Properties: Serialize + Send + Clone,
    {
        let Self {
            layout,
            request,
            inner,
            params,
            ..
        } = self;

        let render_layout = layout.unwrap();
        let ctx = AppContext {
            params,
            request,
            layout: None,
            inner: inner.clone(),
        };

        let layout_html = render_layout(ctx).await;
        let layout_html_string = render_to_static_html(move || layout_html).await;

        let inner = inner.lock().unwrap();
        let links = inner.links.clone();
        let metadata = inner.metadata.clone();
        let scripts = inner.scripts.clone();

        let options = RenderPageOptions {
            layout: layout_html_string,
            metadata,
            links,
            scripts,
        };

        let result_html = render_page_to_html::<COMP, C>(props, options)
            .await
            .unwrap();
        result_html
    }
}

pub struct RenderContext<COMP, C> {
    context: AppContext<C>,
    _marker: PhantomData<COMP>,
}

impl<COMP, C> RenderContext<COMP, C> {
    pub(crate) fn new(context: AppContext<C>) -> Self {
        RenderContext {
            context,
            _marker: PhantomData,
        }
    }
}
impl<COMP, C> RenderContext<COMP, C>
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    pub fn add_metadata(&mut self, metadata: Metadata) {
        self.context.add_metadata(metadata);
    }

    pub fn add_links(&mut self, links: PageLinks) {
        self.context.add_links(links);
    }

    pub fn add_scripts(&mut self, scripts: PageScripts) {
        self.context.add_scripts(scripts);
    }

    pub fn request(&self) -> &Request {
        self.context.request()
    }

    pub fn params(&self) -> &Params {
        self.context.params()
    }
}

impl<COMP, C> RenderContext<COMP, C>
where
    C: BaseComponent<Properties = ChildrenProps>,
    COMP: BaseComponent,
    COMP::Properties: Serialize + Send + Clone,
{
    pub async fn render(self) -> String
    where
        COMP::Properties: Default,
    {
        self.context.render::<COMP>().await
    }

    pub async fn render_with_props(self, props: COMP::Properties) -> String {
        self.context.render_with_props::<COMP>(props).await
    }
}
