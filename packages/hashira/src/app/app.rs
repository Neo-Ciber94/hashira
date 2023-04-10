use super::{
    error_router::{ErrorRouter, ServerErrorRouter},
    router::{PageRouterWrapper, PageRouter},
    AppService, AppServiceInner, BoxFuture, ClientPageRoute, LayoutContext, RenderContext,
    RequestContext, Route,
};
use crate::{
    components::{
        error::{ErrorPage, ErrorPageProps, NotFoundPage},
        RootLayout,
    },
    error::Error,
    web::Response,
};
use http::status::StatusCode;
use route_recognizer::Router;
use serde::de::DeserializeOwned;
use std::{future::Future, marker::PhantomData, rc::Rc, sync::Arc};
use yew::{html::ChildrenProps, BaseComponent, Html};

/// A function that renders the base `index.html`.
pub type RenderLayout = Rc<dyn Fn(LayoutContext) -> BoxFuture<Html>>;

/// A handler for a request.
pub struct PageHandler(
    pub(crate) Box<dyn Fn(RequestContext) -> BoxFuture<Result<Response, Error>>>,
);

impl PageHandler {
    pub fn call(&self, ctx: RequestContext) -> BoxFuture<Result<Response, Error>> {
        (self.0)(ctx)
    }
}

/// A handler for errors.
pub struct ErrorPageHandler(
    pub(crate) Box<dyn Fn(RequestContext, StatusCode) -> BoxFuture<Result<Response, Error>>>,
);

impl ErrorPageHandler {
    pub fn call(
        &self,
        ctx: RequestContext,
        status: StatusCode,
    ) -> BoxFuture<Result<Response, Error>> {
        (self.0)(ctx, status)
    }
}

/// A builder for a `hashira` application.
pub struct App<C> {
    layout: Option<RenderLayout>,
    server_router: Router<Route>,
    page_router: PageRouter,
    client_error_router: ErrorRouter,
    server_error_router: ServerErrorRouter,
    _marker: PhantomData<C>,
}

impl<C> App<C>
where
    C: 'static,
{
    /// Constructs a new empty builder.
    pub fn new() -> Self {
        App {
            layout: None,
            server_router: Router::new(),
            page_router: PageRouter::new(),
            client_error_router: ErrorRouter::new(),
            server_error_router: ServerErrorRouter::new(),
            _marker: PhantomData,
        }
    }

    /// Adds a handler that renders the base `index.html` for the requests.
    ///
    /// By default we use this template:
    /// ```rs
    /// yew::html! {
    ///     <html lang="en">
    ///         <head>
    ///             <Title/>
    ///             <Meta/>
    ///             <Links/>
    ///             <meta charset="utf-8" />
    ///             <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    ///         </head>
    ///         <body>
    ///             <Main>
    ///                 <Content/>
    ///             </Main>
    ///             <Scripts/>
    ///             <LiveReload/>
    ///         </body>
    ///     </html>
    /// }
    /// ```
    pub fn layout<F, Fut>(mut self, layout: F) -> Self
    where
        F: Fn(LayoutContext) -> Fut + 'static,
        Fut: Future<Output = Html> + 'static,
    {
        self.layout = Some(Rc::new(move |ctx| {
            let fut = layout(ctx);
            Box::pin(fut)
        }));
        self
    }

    /// Adds a route handler.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn route(mut self, route: Route) -> Self {
        let path = route.path().to_owned(); // To please the borrow checker
        self.server_router.add(&path, route);
        self
    }

    /// Adds a route handler.
    #[cfg(target_arch = "wasm32")]
    pub fn route(self, _: Route) -> Self {
        self
    }

    /// Adds a page for the given route.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn page<COMP, H, Fut>(mut self, path: &str, handler: H) -> Self
    where
        COMP: BaseComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext<COMP, C>) -> Fut + 'static,
        Fut: Future<Output = Result<Response, Error>> + 'static,
    {
        use super::layout_data::PageLayoutData;

        self.add_component::<COMP>(path);

        self.route(Route::get(
            path,
            PageHandler(Box::new(move |ctx| {
                let layout_data = PageLayoutData::new();
                let render_ctx = RenderContext::new(ctx, layout_data);
                let res = handler(render_ctx);
                Box::pin(res)
            })),
        ))
    }

    /// Adds a page for the given route.
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

    /// Adds an error page for teh given status code.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn error_page<COMP, H, Fut>(mut self, status: StatusCode, handler: H) -> Self
    where
        COMP: BaseComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext<COMP, C>, StatusCode) -> Fut + 'static,
        Fut: Future<Output = Result<Response, Error>> + 'static,
    {
        use super::layout_data::PageLayoutData;

        self.server_error_router.add(
            status,
            ErrorPageHandler(Box::new(move |ctx, status| {
                let layout_data = PageLayoutData::new();
                let render_ctx = RenderContext::new(ctx, layout_data);
                let res = handler(render_ctx, status);
                Box::pin(res)
            })),
        );

        self.add_error_component::<COMP>(status);
        self
    }

    /// Adds an error page for teh given status code.
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
        use super::layout_data::PageLayoutData;

        self.server_error_router
            .add_fallback(ErrorPageHandler(Box::new(move |ctx, status| {
                let layout_data = PageLayoutData::new();
                let render_ctx = RenderContext::new(ctx, layout_data);
                let res = handler(render_ctx, status);
                Box::pin(res)
            })));

        self.add_error_fallback_component::<COMP>();
        self
    }

    /// Adds a default error page to handle all the errors when not matching page error is found.
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
                ctx.add_title(format!(
                    "{} | {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("Page Error")
                ));

                let mut res = ctx.render().await;
                *res.status_mut() = status;
                Ok(res)
            },
        )
        .error_page_fallback(
            move |mut ctx: RenderContext<ErrorPage, C>, status| async move {
                ctx.add_title(format!(
                    "{} | {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("Page Error")
                ));

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

    /// Constructs an `AppService` using this instance.
    pub fn build(self) -> AppService {
        let App {
            layout,
            server_router,
            page_router: client_router,
            client_error_router,
            server_error_router,
            _marker: _,
        } = self;

        let layout = layout.unwrap_or_else(|| Rc::new(render_default_layout));
        let client_router = PageRouterWrapper::from(client_router);
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

        self.page_router.add(
            path,
            ClientPageRoute {
                path: path.to_string(),
                component_id: std::any::type_name::<COMP>().to_owned(),
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
}

fn render_default_layout(_: LayoutContext) -> BoxFuture<yew::Html> {
    Box::pin(async {
        yew::html! {
            <RootLayout/>
        }
    })
}
