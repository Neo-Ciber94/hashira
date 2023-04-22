use super::{
    error_router::{ErrorRouter, ServerErrorRouter},
    router::{PageRouter, PageRouterWrapper},
    AppNested, AppService, AppServiceInner,  ClientPageRoute, LayoutContext,
    RenderContext, RequestContext, Route, AppData, page_head::PageHead,
};
use crate::{
    components::{
        error::{ErrorPage, ErrorPageProps, NotFoundPage},
        id::PageId,
        PageComponent,
    },
    error::Error,
    web::{IntoResponse, Response, Redirect}, routing::PathRouter, types::BoxFuture,
};
use super::PageResponse;
use http::status::StatusCode;
use serde::de::DeserializeOwned;
use std::{future::Future, marker::PhantomData, sync::Arc, pin::Pin};
use yew::{html::ChildrenProps, BaseComponent, Html};

type BoxedFuture<T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'static>>;

/// A function that renders the base `index.html`.
pub type RenderLayout = Arc<dyn Fn(LayoutContext) -> BoxedFuture<Html> + Send + Sync>;

/// A handler for a request.
pub struct PageHandler(pub(crate) Box<dyn Fn(RequestContext) -> BoxFuture<Response> + Send + Sync>);

impl PageHandler {
    pub fn new<H, R, Fut>(handler: H) -> Self
    where
        H: Fn(RequestContext) -> Fut + Send + Sync + 'static,
        R: IntoResponse,
        Fut: Future<Output = R> + Send + Sync + 'static,
    {
        PageHandler(Box::new(move |ctx| {
            let ret = handler(ctx);
            Box::pin(async move {
                let ret = ret.await;
                ret.into_response()
            })
        }))
    }

    pub fn call(&self, ctx: RequestContext) -> BoxFuture<Response> {
        (self.0)(ctx)
    }
}

/// A handler for errors.
#[allow(clippy::type_complexity)]
pub struct ErrorPageHandler(
    pub(crate) Box<dyn Fn(RequestContext, StatusCode) -> BoxFuture<Result<Response, Error>> + Send + Sync>,
);

impl ErrorPageHandler {
    pub fn new<H, Fut>(handler: H) -> Self
    where
        H: Fn(RequestContext, StatusCode) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, Error>> + Send + Sync + 'static,
    {
        ErrorPageHandler(Box::new(move |ctx, status| {
            let fut = handler(ctx, status);
            Box::pin(fut)
        }))
    }

    pub fn call(
        &self,
        ctx: RequestContext,
        status: StatusCode,
    ) -> BoxFuture<Result<Response, Error>> {
        (self.0)(ctx, status)
    }
}

/// A builder for a `hashira` application.
pub struct App<BASE> {
    layout: Option<RenderLayout>,
    server_router: PathRouter<Route>,
    page_router: PageRouter,
    client_error_router: ErrorRouter,
    server_error_router: ServerErrorRouter,
    app_data: AppData,
    _marker: PhantomData<BASE>,

    #[cfg(feature = "hooks")]
    hooks: crate::events::Hooks,
}

impl<BASE> App<BASE> {
    /// Constructs a new empty builder.
    pub fn new() -> Self {
        App {
            layout: None,
            server_router: PathRouter::new(),
            page_router: PageRouter::new(),
            client_error_router: ErrorRouter::new(),
            server_error_router: ServerErrorRouter::new(),
            app_data: Default::default(),
            _marker: PhantomData,

            #[cfg(feature = "hooks")]
            hooks: Default::default(),
        }
    }
}

impl<BASE> App<BASE>
where
    BASE: 'static,
{
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
        F: Fn(LayoutContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Html> + Send + Sync + 'static,
    {
        self.layout = Some(Arc::new(move |ctx| {
            let fut = layout(ctx);
            Box::pin(fut)
        }));
        self
    }

    /// Adds a route handler.
    #[cfg_attr(client="client", allow(unused_mut, unused_variables))]
    pub fn route(mut self, route: Route) -> Self {
        #[cfg(not(client = "client"))]
        {
            let path = route.path().to_owned(); // To please the borrow checker
            self.server_router.insert(&path, route).expect("failed to add route");
        }

        self
    }

    /// Adds nested routes for the given path.
    pub fn nest(mut self, base_path: &str, scope: AppNested<BASE>) -> Self {
        crate::routing::assert_valid_route(base_path).expect("invalid base path");

        #[cfg(not(feature = "client"))]
        {
            for (sub, route) in scope.server_router {
                let path = if sub == "/" {
                        base_path.to_owned()
                    } else {
                        format!("{base_path}{sub}")
                    };

                self.server_router.insert(&path, route).expect("failed to add route");
            }
        }

        for (sub, route) in scope.page_router {
            let path = if sub == "/" {
                    base_path.to_owned()
                } else {
                    format!("{base_path}{sub}")
                };
        
            self.page_router.insert(&path, route);
        }

        self
    }

    /// Adds a page for the given route.
    #[cfg_attr(feature = "client", allow(unused_variables))]
    pub fn page<COMP, H, Fut>(mut self, path: &str, handler: H) -> Self
    where
        COMP: PageComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<PageResponse<COMP, BASE>, Error>> + Send + Sync + 'static,
    {
        self.add_component::<COMP>(path);

        #[cfg(not(feature = "client"))]
        {
            self.route(Route::get(path, move |ctx| {
                let head = super::page_head::PageHead::new();
                let render_layout = ctx.app_data::<RenderLayout>().cloned().unwrap();
                let render_ctx = RenderContext::new(ctx, head, render_layout);
                
                // Returns the future
                handler(render_ctx)
            }))
        }

        // We don't add pages in the client
        #[cfg(feature = "client")]
        self
    }

    /// Adds an error page for teh given status code.
    #[cfg_attr(feature="client", allow(unused_variables))]
    pub fn error_page<COMP, H, Fut>(mut self, status: StatusCode, handler: H) -> Self
    where
        COMP: PageComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext, StatusCode) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<PageResponse<COMP, BASE>, Error>> + Send + Sync + 'static,
    {
        #[cfg(not(feature = "client"))]
        {
            use futures::TryFutureExt;

            self.server_error_router.insert(
                status,
                ErrorPageHandler::new(move |ctx, status| {
                    let head = super::page_head::PageHead::new();
                    let render_layout = ctx.app_data::<RenderLayout>().cloned().unwrap();
                    let render_ctx = RenderContext::new(ctx, head, render_layout);

                    // Returns the future
                    handler(render_ctx, status).map_ok(|x| x.into_response())
                }),
            );
        }

        self.add_error_component::<COMP>(status);
        self
    }

    /// Register a page to handle any error.
    #[cfg_attr(feature="client", allow(unused_variables))]
    pub fn error_page_fallback<COMP, H, Fut>(mut self, handler: H) -> Self
    where
        COMP: PageComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext, StatusCode) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<PageResponse<COMP, BASE>, Error>> + Send + Sync + 'static,
    {
        #[cfg(not(feature = "client"))]
        {
            use futures::TryFutureExt;

            self.server_error_router
                .fallback(ErrorPageHandler(Box::new(move |ctx, status| {
                    let head = super::page_head::PageHead::new();
                    let render_layout = ctx.app_data::<RenderLayout>().cloned().unwrap();
                    let render_ctx = RenderContext::new(ctx, head, render_layout);
                    let res = handler(render_ctx, status).map_ok(|x| x.into_response());
                    Box::pin(res)
                })));
        }

        self.add_error_fallback_component::<COMP>();
        self
    }

    /// Adds the default `404` error page and a fallback error page.
    pub fn use_default_error_pages(self) -> Self
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
    {
        self.error_page(
            StatusCode::NOT_FOUND,
            move |mut ctx: RenderContext, status: StatusCode| async move {
                ctx.title(format!(
                    "{} | {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("Not Found")
                ));

                let mut res = ctx.render::<NotFoundPage, BASE>().await;
                *res.status_mut() = status;
                Ok(res)
            },
        )
        .error_page_fallback(
            move |mut ctx: RenderContext, status| async move {
                ctx.title(format!(
                    "{} | {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("Page Error")
                ));

                let mut res = ctx
                    .render_with_props::<ErrorPage, BASE>(ErrorPageProps {
                        status,
                        message: None,
                    })
                    .await;
                *res.status_mut() = status;
                Ok(res)
            },
        )
    }

    /// Adds a shared state that will be shared between server and client.
    pub fn app_data<T>(mut self, data: T) -> Self where T: Send + Sync + 'static {
        self.app_data.insert::<T>(data);
        self
    }

    /// Adds a shared state that will be available on the server.
    #[cfg(not(feature="wasm32"))]
    pub fn server_data<T>(self, data: T) -> Self where T: Send + Sync + 'static {
        self.app_data(data)
    }

    /// Adds a shared state that will be available on the server.
    #[cfg(feature="wasm32")]
    pub fn server_data<T>(self, _: T) -> Self where T: Send + Sync + 'static {
        self
    }

    /// Adds a shared state that will be available on the client.
    #[cfg(feature="wasm32")]
    pub fn client_data<T>(self, data: T) -> Self where T: Send + Sync + 'static {
        self.app_data(data)
    }

    /// Adds a shared state that will be available on the client.
    #[cfg(not(feature="wasm32"))]
    pub fn client_data<T>(self, _: T) -> Self where T: Send + Sync + 'static {
        self
    }

    /// Adds the given `Hooks`.
    #[cfg(feature = "hooks")]
    pub fn hooks(mut self, hooks: crate::events::Hooks) -> Self {
        self.hooks.extend(hooks);
        self
    }

    /// Constructs an `AppService` using this instance.
    pub fn build(self) -> AppService where BASE: BaseComponent<Properties =ChildrenProps>{
        let App {
            layout,
            server_router,
            page_router: client_router,
            client_error_router,
            server_error_router,
            mut app_data,
            _marker: _,

            #[cfg(feature = "hooks")]
            hooks,
        } = self;

        let layout = layout.unwrap_or_else(|| {
            // Pass the default layout
            let render_layout =
                |ctx| Box::pin(crate::components::root_layout(ctx)) as BoxedFuture<yew::Html>;

            Arc::new(render_layout)
        });

        // Add startup app data
        app_data.insert::<RenderLayout>(layout); // The RenderContext require the RenderLayout

        // Construct app service
        let client_router = PageRouterWrapper::from(client_router);
        let client_error_router = Arc::from(client_error_router);
        let app_data = Arc::new(app_data);
        let inner = AppServiceInner {
            app_data,
            server_router,
            client_router,
            client_error_router,
            server_error_router,

            #[cfg(feature = "hooks")]
            hooks: Arc::new(hooks),
        };

        AppService::new(Arc::new(inner))
    }

    fn add_component<COMP>(&mut self, path: &str)
    where
        COMP: PageComponent,
        COMP::Properties: DeserializeOwned,
    {
        use crate::components::AnyComponent;
        
        log::debug!(
            "Registering component `{}` on {path}",
            std::any::type_name::<COMP>()
        );

        self.page_router.insert(
            path,
            ClientPageRoute {
                path: path.to_string(),
                page_id: PageId::of::<COMP>(),
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
        COMP: PageComponent,
        COMP::Properties: DeserializeOwned,
    {
        use crate::components::AnyComponent;

        log::debug!(
            "Registering error component `{}` for {status}",
            std::any::type_name::<COMP>()
        );

        self.client_error_router.insert(
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
            .fallback(AnyComponent::<serde_json::Value>::new(|props_json| {
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

impl<BASE> Default for App<BASE> {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a redirection route.
/// 
/// # Panic
/// - If the status code is not a redirection
/// - The from/to are invalid uri
pub fn redirect(from: &str, to: &str, status: StatusCode) -> Route {
    let to = to.to_owned();
    Route::any(from, move |_| {
        let to = to.clone();
        async move {       
            Redirect::new(to, status).expect("invalid redirection")
        }
    })
}

/// Returns a route that renders the given component with the given `<head>`.
pub fn render<COMP, BASE>(route: &str, head: PageHead) -> Route     
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: Default + serde::Serialize + Send + Sync + Clone {
    Route::get(route, move |ctx| {
        let head = head.clone();
        let render_layout = ctx.app_data::<RenderLayout>().cloned().unwrap();
        let render_ctx = RenderContext::new(ctx, head, render_layout);

        // Returns the future
        render_ctx.render::<COMP, BASE>()
    })
}

/// Returns a route that renders the given component with props, with the given `<head>`.
pub fn render_with_props<COMP, BASE>(route: &str, props: COMP::Properties, head: PageHead) -> Route     
    where
        BASE: BaseComponent<Properties = ChildrenProps>,
        COMP: PageComponent,
        COMP::Properties: serde::Serialize + Send + Sync + Clone {
    Route::get(route, move |ctx| {
        let head = head.clone();
        let props = props.clone();

        let render_layout = ctx.app_data::<RenderLayout>().cloned().unwrap();
        let render_ctx = RenderContext::new(ctx, head, render_layout);

        // Returns the future
        render_ctx.render_with_props::<COMP, BASE>(props)
    })
}