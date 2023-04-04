use super::{
    error_router::{ErrorRouter, ServerErrorRouter},
    router::ClientRouter,
    AppService, AppServiceInner, BoxFuture, ClientPageRoute, RenderContext, RequestContext, Route,
};
use crate::{
    components::{
        error::{ErrorPage, ErrorPageProps, NotFoundPage},
        RootLayout,
    },
    error::Error,
    server::Metadata,
    web::Response,
};
use http::status::StatusCode;
use route_recognizer::Router;
use serde::de::DeserializeOwned;
use std::{future::Future, rc::Rc, sync::Arc};
use yew::{html::ChildrenProps, BaseComponent, Html};

pub type RenderLayout<C> = Rc<dyn Fn(RequestContext<C>) -> BoxFuture<Html>>;

pub struct PageHandler<C>(
    pub(crate) Box<dyn Fn(RequestContext<C>) -> BoxFuture<Result<Response, Error>>>,
);

impl<C> PageHandler<C> {
    pub fn call(&self, ctx: RequestContext<C>) -> BoxFuture<Result<Response, Error>> {
        (self.0)(ctx)
    }
}

pub struct ErrorPageHandler<C>(
    pub(crate) Box<dyn Fn(RequestContext<C>, StatusCode) -> BoxFuture<Result<Response, Error>>>,
);

impl<C> ErrorPageHandler<C> {
    pub fn call(
        &self,
        ctx: RequestContext<C>,
        status: StatusCode,
    ) -> BoxFuture<Result<Response, Error>> {
        (self.0)(ctx, status)
    }
}

pub struct App<C> {
    layout: Option<RenderLayout<C>>,
    server_router: Router<Route<C>>,
    client_router: Router<ClientPageRoute>,
    client_error_router: ErrorRouter,
    server_error_router: ServerErrorRouter<C>,
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
            client_error_router: ErrorRouter::new(),
            server_error_router: ServerErrorRouter::new(),
        }
    }

    pub fn layout<F, Fut>(mut self, layout: F) -> Self
    where
        F: Fn(RequestContext<C>) -> Fut + 'static,
        Fut: Future<Output = Html> + 'static,
    {
        self.layout = Some(Rc::new(move |ctx| {
            let html = layout(ctx);
            Box::pin(html)
        }));
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn route(mut self, route: Route<C>) -> Self {
        let path = route.path().to_owned(); // To please the borrow checker
        self.server_router.add(&path, route);
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn route(self, _: Route<C>) -> Self {
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn page<COMP, H, Fut>(mut self, path: &str, handler: H) -> Self
    where
        COMP: BaseComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext<COMP, C>) -> Fut + 'static,
        Fut: Future<Output = Result<Response, Error>> + 'static,
    {
        self.add_component::<COMP>(path);

        self.route(Route::get(
            path,
            PageHandler(Box::new(move |ctx| {
                let render_ctx = RenderContext::new(ctx);
                let res = handler(render_ctx);
                Box::pin(res)
            })),
        ))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn page<COMP, H, Fut>(mut self, path: &str, _: H) -> Self
    where
        COMP: BaseComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext<COMP, C>) -> Fut + 'static,
        Fut: Future<Output = Result<Response, Error>> + 'static,
    {
        self.add_component::<COMP>(path);
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn error_page<COMP, H, Fut>(mut self, status: StatusCode, handler: H) -> Self
    where
        COMP: BaseComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext<COMP, C>, StatusCode) -> Fut + 'static,
        Fut: Future<Output = Result<Response, Error>> + 'static,
    {
        self.server_error_router.add(
            status,
            ErrorPageHandler(Box::new(move |ctx, status| {
                let render_ctx = RenderContext::new(ctx);
                let res = handler(render_ctx, status);
                Box::pin(res)
            })),
        );

        self.add_error_component::<COMP>(status);
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn error_page<COMP, H, Fut>(mut self, status: StatusCode, _: H) -> Self
    where
        COMP: BaseComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext<COMP, C>, StatusCode) -> Fut + 'static,
        Fut: Future<Output = Result<Response, Error>> + 'static,
    {
        self.add_error_component::<COMP>(status);
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn error_page_fallback<COMP, H, Fut>(mut self, handler: H) -> Self
    where
        COMP: BaseComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext<COMP, C>, StatusCode) -> Fut + 'static,
        Fut: Future<Output = Result<Response, Error>> + 'static,
    {
        self.server_error_router
            .add_fallback(ErrorPageHandler(Box::new(move |ctx, status| {
                let render_ctx = RenderContext::new(ctx);
                let res = handler(render_ctx, status);
                Box::pin(res)
            })));

        self.add_error_fallback_component::<COMP>();
        self
    }

    #[cfg(target_arch = "wasm32")]
    pub fn error_page_fallback<COMP, H, Fut>(mut self, _: H) -> Self
    where
        COMP: BaseComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext<COMP, C>, StatusCode) -> Fut + 'static,
        Fut: Future<Output = Result<Response, Error>> + 'static,
    {
        self.add_error_fallback_component::<COMP>();
        self
    }

    /// Adds the default `404` error page and a fallback error page.
    pub fn use_default_error_pages(self) -> Self
    where
        C: BaseComponent<Properties = ChildrenProps>,
    {
        self.error_page(
            StatusCode::NOT_FOUND,
            move |mut ctx: RenderContext<NotFoundPage, C>, status: StatusCode| async move {
                ctx.add_metadata(Metadata::new().title(format!(
                    "{} | {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("Page Error")
                )));

                let mut res = ctx.render().await;
                *res.status_mut() = status;
                Ok(res)
            },
        )
        .error_page_fallback(
            move |mut ctx: RenderContext<ErrorPage, C>, status| async move {
                ctx.add_metadata(Metadata::new().title(format!(
                    "{} | {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("Page Error")
                )));

                let mut res = ctx
                    .render_with_props(ErrorPageProps {
                        status,
                        message: None,
                    })
                    .await;
                *res.status_mut() = status;
                Ok(res)
            },
        )
    }

    fn add_component<COMP>(&mut self, path: &str)
    where
        COMP: BaseComponent,
        COMP::Properties: DeserializeOwned,
    {
        use crate::components::AnyComponent;

        log::debug!(
            "Registering component `{}` on {path}",
            std::any::type_name::<COMP>()
        );

        self.client_router.add(
            path,
            ClientPageRoute {
                path: path.to_string(),
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

    fn add_error_component<COMP>(&mut self, status: StatusCode)
    where
        COMP: BaseComponent,
        COMP::Properties: DeserializeOwned,
    {
        use crate::components::AnyComponent;

        log::debug!(
            "Registering error component `{}` for {status}",
            std::any::type_name::<COMP>()
        );

        self.client_error_router.add(
            status,
            AnyComponent::<serde_json::Value>::new(|props_json| {
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
        );
    }

    fn add_error_fallback_component<COMP>(&mut self)
    where
        COMP: BaseComponent,
        COMP::Properties: DeserializeOwned,
    {
        use crate::components::AnyComponent;

        log::debug!(
            "Registering fallback error component `{}`",
            std::any::type_name::<COMP>()
        );

        self.client_error_router
            .add_fallback(AnyComponent::<serde_json::Value>::new(|props_json| {
                let props = serde_json::from_value(props_json).unwrap_or_else(|err| {
                    panic!(
                        "Failed to deserialize `{}` component props. {err}",
                        std::any::type_name::<COMP>()
                    )
                });

                yew::html! {
                    <COMP ..props/>
                }
            }));
    }

    pub fn build(self) -> AppService<C> {
        let App {
            layout,
            server_router,
            client_router,
            client_error_router,
            server_error_router,
        } = self;

        let layout = layout.unwrap_or_else(|| Rc::new(render_default_layout));
        let client_router = ClientRouter::from(client_router);
        let client_error_router = Arc::from(client_error_router);
        let inner = AppServiceInner {
            layout,
            server_router,
            client_router,
            client_error_router,
            server_error_router,
        };

        AppService::new(Rc::new(inner))
    }
}

fn render_default_layout<C>(_: RequestContext<C>) -> BoxFuture<yew::Html> {
    Box::pin(async {
        yew::html! {
            <RootLayout/>
        }
    })
}
