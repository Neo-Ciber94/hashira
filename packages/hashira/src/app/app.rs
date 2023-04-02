use super::{
    client_router::ClientRouter, AppContext, AppService, BoxFuture, ClientPageRoute, Inner,
    RenderContext, ServerPageRoute,
};
use crate::{components::RootLayout, web::Response};
use route_recognizer::Router;
use serde::de::DeserializeOwned;
use std::{future::Future, rc::Rc};
use yew::{BaseComponent, Html};

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

    #[cfg(not(target_arch = "wasm32"))]
    pub fn page<COMP, H, Fut>(mut self, path: &str, handler: H) -> Self
    where
        COMP: BaseComponent,
        COMP::Properties: DeserializeOwned,
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
        self.add_client_page::<COMP, H, Fut>(path);
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn page<COMP, H, Fut>(mut self, path: &str, _: H) -> Self
    where
        COMP: BaseComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext<COMP, C>) -> Fut + 'static,
        Fut: Future<Output = Response> + 'static,
    {
        self.add_client_page::<COMP, H, Fut>(path);
        self
    }

    fn add_client_page<COMP, H, Fut>(&mut self, path: &str)
    where
        COMP: BaseComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext<COMP, C>) -> Fut + 'static,
        Fut: Future<Output = Response> + 'static,
    {
        use crate::components::AnyComponent;

        log::info!(
            "Registering component `{}` on {path}",
            std::any::type_name::<COMP>()
        );

        self.client_router.add(
            path,
            ClientPageRoute {
                match_pattern: path.to_string(),
                component: AnyComponent::<serde_json::Value>::new(|props_json| {
                    let props = serde_json::from_value(props_json).unwrap_or_else(|err| {
                        panic!(
                            "Failed to deserialize `{}` component props. {err}",
                            std::any::type_name::<COMP>()
                        )
                    });

                    yew::html! {
                        <COMP ..props/>
                    }
                }),
            },
        );
    }

    pub fn build(self) -> AppService<C> {
        let App {
            layout,
            server_router,
            client_router,
        } = self;

        let layout = layout.unwrap_or_else(|| Rc::new(render_default_layout));
        let client_router = ClientRouter::from(client_router);
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
            <RootLayout/>
        }
    })
}
