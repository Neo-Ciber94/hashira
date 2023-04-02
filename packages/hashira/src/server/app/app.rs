use super::{
    AppContext, AppService, BoxFuture, ClientPageRoute, Inner, RenderContext, ServerPageRoute,
};
use crate::{server::DefaultLayout, web::Response};
use route_recognizer::Router;
use std::{future::Future, rc::Rc};
use yew::{Html, BaseComponent};

pub type RenderLayout<C> = Rc<dyn Fn(AppContext<C>) -> BoxFuture<Html>>;

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

    pub fn page<COMP, H, Fut>(mut self, path: &str, handler: H) -> Self
    where
        COMP: BaseComponent,
        H: Fn(RenderContext<COMP, C>) -> Fut + 'static,
        Fut: Future<Output = Response> + 'static,
    {
        assert!(path.starts_with("/"), "page path must start with `/`");

        let page = ServerPageRoute {
            match_pattern: path.to_string(),
            handler: PageHandler(Box::new(move |ctx| {
                let render_ctx = RenderContext::new(ctx);
                let res = handler(render_ctx);
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
