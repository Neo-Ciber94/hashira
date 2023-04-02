use super::{AppService, BoxFuture, ClientPageRoute, Inner, ServerPageRoute};
use crate::server::{Metadata, PageLinks, PageScripts};
use crate::{
    server::{render_page_to_html, render_to_static_html, DefaultLayout, RenderPageOptions},
    web::{Request, Response},
};
use route_recognizer::{Params, Router};
use serde::Serialize;
use std::{
    future::Future,
    rc::Rc,
    sync::{Arc, Mutex},
};
use yew::{html::ChildrenProps, BaseComponent, Html};

pub type RenderLayout<C> = Rc<dyn Fn(AppContext<C>) -> BoxFuture<Html>>;

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

pub struct PageHandler<C>(pub(crate) Box<dyn Fn(AppContext<C>) -> BoxFuture<Response>>);

impl<C> PageHandler<C> {
    pub fn call(&self, ctx: AppContext<C>) -> BoxFuture<Response> {
        (self.0)(ctx)
    }
}

pub struct App<C> {
    layout: Option<RenderLayout<C>>,
    server_router: Router<ServerPageRoute<C>>,
    client_router: Router<ClientPageRoute>,
}

impl<C> App<C>
where
    C: 'static,
{
    pub fn new() -> Self {
        App {
            layout: None,
            server_router: Router::new(),
            client_router: Router::new(),
        }
    }

    pub fn layout<F, Fut>(mut self, layout: F) -> Self
    where
        F: Fn(AppContext<C>) -> Fut + 'static,
        Fut: Future<Output = Html> + 'static,
    {
        self.layout = Some(Rc::new(move |ctx| {
            let html = layout(ctx);
            Box::pin(html)
        }));
        self
    }

    pub fn page<H, Fut>(mut self, path: &str, handler: H) -> Self
    where
        H: Fn(AppContext<C>) -> Fut + 'static,
        Fut: Future<Output = Response> + 'static,
    {
        assert!(path.starts_with("/"), "page path must start with `/`");

        let page = ServerPageRoute {
            match_pattern: path.to_string(),
            handler: PageHandler(Box::new(move |ctx| {
                let res = handler(ctx);
                Box::pin(res)
            })),
        };

        self.server_router.add(path, page);
        self
    }

    pub fn build(self) -> AppService<C> {
        let App {
            layout,
            server_router,
            client_router,
        } = self;

        let layout = layout.unwrap_or_else(|| Rc::new(render_default_layout));
        let inner = Inner {
            layout,
            server_router,
            client_router,
        };

        AppService::new(Rc::new(inner))
    }
}

fn render_default_layout<C>(_: AppContext<C>) -> BoxFuture<yew::Html> {
    Box::pin(async {
        yew::html! {
            <DefaultLayout/>
        }
    })
}
