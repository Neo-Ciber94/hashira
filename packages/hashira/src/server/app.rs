use super::{Metadata, PageLinks, PageScripts};
use crate::{
    server::{render_page_to_html, render_to_static_html, DefaultLayout, RenderPageOptions},
    web::{Request, Response},
};
use route_recognizer::{Params, Router};
use serde::Serialize;
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
    sync::{Arc, Mutex},
};
use yew::{html::ChildrenProps, BaseComponent, Html};

type BoxFuture<T> = Pin<Box<dyn Future<Output = T>>>;

type RenderLayout<C> = Rc<dyn Fn(AppContext<C>) -> BoxFuture<Html>>;

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
    _marker: PhantomData<C>,
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
            _marker: PhantomData,
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
            _marker: PhantomData,
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
            _marker: PhantomData,
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

pub struct PageHandler<C>(Box<dyn Fn(AppContext<C>) -> BoxFuture<Response>>);

impl<C> PageHandler<C> {
    pub fn call(&self, ctx: AppContext<C>) -> BoxFuture<Response> {
        (self.0)(ctx)
    }
}

pub struct App<C> {
    layout: Option<RenderLayout<C>>,
    router: Router<PageHandler<C>>,
}

impl<C> App<C>
where
    C: 'static,
{
    pub fn new() -> Self {
        App {
            layout: None,
            router: Router::new(),
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

    pub fn build(self) -> AppService<C> {
        let App { layout, router } = self;
        let layout = layout.unwrap_or_else(|| Rc::new(render_default_layout));
        let inner = Inner { layout, router };

        AppService {
            inner: Rc::new(inner),
        }
    }
}

struct Inner<C> {
    pub(crate) layout: RenderLayout<C>,
    pub(crate) router: Router<PageHandler<C>>,
}

pub struct AppService<C> {
    inner: Rc<Inner<C>>,
}

impl<C> AppService<C> {
    /// Create a context to be used in the request.
    pub fn create_context(&self, request: Request, params: Params) -> AppContext<C> {
        let layout = self.inner.layout.clone();
        AppContext::new(request, layout, params)
    }

    /// Returns the router with all the pages.
    pub fn router(&self) -> &Router<PageHandler<C>> {
        &self.inner.router
    }

    /// Process the incoming request and return the response.
    pub async fn handle(&self, req: Request, path: &str) -> Response {
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

impl<C> Clone for AppService<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

fn render_default_layout<C>(_: AppContext<C>) -> BoxFuture<yew::Html> {
    Box::pin(async {
        yew::html! {
            <DefaultLayout/>
        }
    })
}
