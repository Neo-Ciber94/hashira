use super::{Metadata, PageLinks, PageScripts};
use crate::server::{render_page_to_html, render_to_string, DefaultLayout, RenderPageOptions};
use route_recognizer::Router;
use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{Arc, Mutex},
};
use yew::{BaseComponent, Html};

type LocalBoxFuture<T> = Pin<Box<dyn Future<Output = T>>>;

type RenderLayout<Req, Res> = Rc<dyn Fn(AppContext<Req, Res>) -> LocalBoxFuture<Html>>;

struct AppContextInner {
    // The `<meta>` tags of the page to render
    metadata: Metadata,

    // the <link> tags of the page to render
    links: PageLinks,

    // the <script> tags of the page to render
    scripts: PageScripts,
}

pub struct AppContext<Req, Res> {
    layout: Option<RenderLayout<Req, Res>>,
    request: Req,
    inner: Arc<Mutex<AppContextInner>>,
}

impl<Req, Res> AppContext<Req, Res> {
    pub fn new(request: Req, layout: RenderLayout<Req, Res>) -> Self {
        let inner = AppContextInner {
            metadata: Metadata::default(),
            links: PageLinks::default(),
            scripts: PageScripts::default(),
        };

        AppContext {
            request,
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
        &self.request
    }

    pub async fn render<COMP>(self) -> String
    where
        COMP: BaseComponent,
        COMP::Properties: Default + Send + Clone,
    {
        let props = COMP::Properties::default();
        self.render_with_props::<COMP>(props).await
    }

    pub async fn render_with_props<COMP>(self, props: COMP::Properties) -> String
    where
        COMP: BaseComponent,
        COMP::Properties: Send + Clone,
    {
        let Self {
            layout,
            request,
            inner,
            ..
        } = self;

        let render_layout = layout.unwrap();
        let ctx = AppContext {
            request,
            layout: None,
            inner: inner.clone(),
        };

        let layout_html = render_layout(ctx).await;
        let layout_html_string = render_to_string(move || layout_html).await;

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

pub struct PageHandler<Req, Res>(Box<dyn Fn(AppContext<Req, Res>) -> LocalBoxFuture<Res>>);

impl<Req, Res> PageHandler<Req, Res> {
    pub fn call(&self, ctx: AppContext<Req, Res>) -> LocalBoxFuture<Res> {
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
    pub fn create_context(&self, request: Req) -> AppContext<Req, Res> {
        let layout = self.inner.layout.clone();
        AppContext::new(request, layout)
    }

    pub fn router(&self) -> &Router<PageHandler<Req, Res>> {
        &self.inner.router
    }
}

impl<Req, Res> Clone for AppService<Req, Res> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

fn render_default_layout<Req, Res>(_: AppContext<Req, Res>) -> LocalBoxFuture<yew::Html>
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
/*
### Server adapter:

|req: Request| {
    let path = req.path();
    let service = req.data::<AppService>().unwrap();
    let page = service.router.recognize(path).unwrap();
    let ctx = service.create_context(req);
    let res = page.call(ctx).await;
    Ok(res)
}


### Page handle

|ctx: AppContext| {
    ctx.add_metadata(...);
    ctx.add_links(...);
    ctx.add_scripts(...);

    let req = ctx.request();
    let id = req.params.get::<u32>("id");
    let user = db.get_user_by_id(id).await.unwrap();
    let res = ctx.render_with_props<Component>(user).await;
    Ok(res)
}
*/
