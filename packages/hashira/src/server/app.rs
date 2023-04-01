use super::{Metadata, PageLinks, PageScripts};
use crate::server::{render_page_to_html, render_to_static_html, DefaultLayout, RenderPageOptions};
use route_recognizer::{Params, Router};
use serde::Serialize;
use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{Arc, Mutex},
};
use yew::{BaseComponent, Html};

type BoxFuture<T> = Pin<Box<dyn Future<Output = T>>>;

type RenderLayout<Req, Res> = Rc<dyn Fn(AppContext<Req, Res>) -> BoxFuture<Html>>;

struct AppContextInner {
    // The `<meta>` tags of the page to render
    metadata: Metadata,

    // the <link> tags of the page to render
    links: PageLinks,

    // the <script> tags of the page to render
    scripts: PageScripts,
}

pub struct AppContext<Req, Res> {
    request: Option<Req>,
    params: Params,
    layout: Option<RenderLayout<Req, Res>>,
    inner: Arc<Mutex<AppContextInner>>,
}

impl<Req, Res> AppContext<Req, Res> {
    pub fn new(request: Req, layout: RenderLayout<Req, Res>, params: Params) -> Self {
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

    pub(crate) fn no_request(layout: RenderLayout<Req, Res>, params: Params) -> Self {
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

impl<Req, Res> AppContext<Req, Res> {
    pub fn add_metadata(&mut self, metadata: Metadata) {
        self.inner.lock().unwrap().metadata.extend(metadata);
    }

    pub fn add_links(&mut self, links: PageLinks) {
        self.inner.lock().unwrap().links.extend(links);
    }

    pub fn add_scripts(&mut self, scripts: PageScripts) {
        self.inner.lock().unwrap().scripts.extend(scripts);
    }

    pub fn request(&self) -> &Req {
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

        let result_html = render_page_to_html::<COMP>(props, options).await.unwrap();
        result_html
    }
}

pub struct PageHandler<Req, Res>(Box<dyn Fn(AppContext<Req, Res>) -> BoxFuture<Res>>);

impl<Req, Res> PageHandler<Req, Res> {
    pub fn call(&self, ctx: AppContext<Req, Res>) -> BoxFuture<Res> {
        (self.0)(ctx)
    }
}

pub struct App<Req, Res> {
    layout: Option<RenderLayout<Req, Res>>,
    router: Router<PageHandler<Req, Res>>,
}

impl<Req, Res> App<Req, Res>
where
    Req: 'static,
    Res: 'static,
{
    pub fn new() -> Self {
        App {
            layout: None,
            router: Router::new(),
        }
    }

    pub fn layout<F, Fut>(mut self, layout: F) -> Self
    where
        F: Fn(AppContext<Req, Res>) -> Fut + 'static,
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
        H: Fn(AppContext<Req, Res>) -> Fut + 'static,
        Fut: Future<Output = Res> + 'static,
    {
        assert!(path.starts_with("/"));
        self.router.add(
            path,
            PageHandler(Box::new(move |ctx| {
                let res = handler(ctx);
                Box::pin(res)
            })),
        );
        self
    }

    pub fn build(self) -> AppService<Req, Res> {
        let App { layout, router } = self;
        let layout = layout.unwrap_or_else(|| Rc::new(render_default_layout));
        let inner = Inner { layout, router };
        AppService {
            inner: Rc::new(inner),
        }
    }
}

struct Inner<Req, Res> {
    pub(crate) layout: RenderLayout<Req, Res>,
    pub(crate) router: Router<PageHandler<Req, Res>>,
}

pub struct AppService<Req, Res> {
    inner: Rc<Inner<Req, Res>>,
}

impl<Req, Res> AppService<Req, Res> {
    /// Create a context to be used in the request.
    pub fn create_context(&self, request: Req, params: Params) -> AppContext<Req, Res> {
        let layout = self.inner.layout.clone();
        AppContext::new(request, layout, params)
    }

    /// Returns the router with all the pages.
    pub fn router(&self) -> &Router<PageHandler<Req, Res>> {
        &self.inner.router
    }

    /// Process the incoming request and return the response.
    pub async fn handle(&self, req: Req, path: &str) -> Res {
        match self.inner.router.recognize(&path) {
            Ok(mtch) => {
                let params = mtch.params().clone();
                let ctx = self.create_context(req, params);
                let res = mtch.handler().call(ctx).await;
                res
            }
            Err(_) => {
                todo!("Return a 404 component")
            }
        }
    }

    /// Returns the `html` template of the layout
    pub async fn get_layout_html(&self) -> String {
        let layout = self.inner.layout.clone();
        let params = Params::new();
        let ctx = AppContext::no_request(layout, params);
        let render_layout = &self.inner.layout;
        let layout_html = render_layout(ctx).await;
        let html_string = render_to_static_html(move || layout_html).await;
        html_string
    }
}

impl<Req, Res> Clone for AppService<Req, Res> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

fn render_default_layout<Req, Res>(_: AppContext<Req, Res>) -> BoxFuture<yew::Html>
where
    Req: 'static,
    Res: 'static,
{
    Box::pin(async {
        yew::html! {
            <DefaultLayout/>
        }
    })
}
