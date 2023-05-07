use super::{
    error_router::{ErrorRouter, ServerErrorRouter},
    router::{PageRouter, PageRouterWrapper},
    AppNested, AppService, AppServiceInner, LayoutContext,
    RequestContext, AppData, DefaultHeaders, Handler, ResponseError,
};
use crate::{
    components::{
        error::{ErrorPage, NotFoundPage},
        id::PageId,
        PageComponent,
    },
    routing::{Route, ClientPageRoute},
    error::Error,
    web::{IntoResponse, Response, Redirect, FromRequest}, routing::PathRouter, types::BoxFuture, actions::Action,
};

use http::{status::StatusCode, HeaderMap};
use serde::de::DeserializeOwned;
use std::{future::Future, marker::PhantomData, sync::Arc, pin::Pin};
use yew::{html::ChildrenProps, BaseComponent, Html};



type BoxedFuture<T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'static>>;

/// A function that renders the base `index.html`.
pub type RenderLayout = Arc<dyn Fn(LayoutContext) -> BoxedFuture<Html> + Send + Sync>;

/// A handler for a request.
pub struct PageHandler(pub(crate) Box<dyn Fn(RequestContext) -> BoxFuture<Response> + Send + Sync>);

impl PageHandler {
    pub fn new<H, Args>(handler: H) -> Self
    where
        Args: FromRequest + Send + 'static,
        H: Handler<Args> + Sync + Send,
        H::Future: Future + Send + 'static,
        H::Output: IntoResponse,
        <Args as FromRequest>::Fut : Send
    {
        PageHandler(Box::new(move |ctx| {      
            let handler = handler.clone();
            Box::pin(async move {
                let args = match Args::from_request(&ctx).await {
                    Ok(x) => x,
                    Err(err) => {
                        return ResponseError::with_error(err).into_response();
                    }
                };
                let ret = handler.call(args).await;
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
        Fut: Future<Output = Result<Response, Error>> + Send + 'static,
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
    default_headers: HeaderMap,
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
            default_headers: Default::default(),
            _marker: PhantomData,

            #[cfg(feature = "hooks")]
            hooks: Default::default(),
        }
    }
}

impl<BASE> App<BASE>
where
    BASE: BaseComponent<Properties = ChildrenProps> + 'static, 
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
            log::debug!("Registering route: {}", route.path());
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
            use super::IsBaseRoute;

            for (sub, route) in scope.server_router {
                let path = match sub.as_str() {
                    "/" => base_path.to_owned(),
                    _ if route.extensions().get::<IsBaseRoute>().is_some() => sub.to_owned(),
                    _ => format!("{base_path}{sub}")
                };
                
                log::debug!("Registering route: {path}");
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
    pub fn page<COMP>(mut self) -> Self
    where
        COMP: PageComponent,
        COMP::Properties: DeserializeOwned,
    {
        let route = COMP::route().unwrap_or_else(|| panic!("`{}` is not declaring a route", std::any::type_name::<COMP>()));
        self.add_component::<COMP>(route);

        #[cfg(not(feature = "client"))]
        {
            use crate::app::RenderContext;

            self.route(Route::get(route, move |ctx: RequestContext| {
                let head = super::page_head::PageHead::new();
                let render_layout = ctx.app_data::<RenderLayout>().cloned().unwrap();
                let render_ctx = RenderContext::new(ctx, head, render_layout);

                // Returns the future
                COMP::render::<BASE>(render_ctx)
            }))
        }

        // We don't add pages in the client
        #[cfg(feature = "client")]
        self
    }

    /// Adds an error page for teh given status code.
    #[cfg_attr(feature="client", allow(unused_variables))]
    pub fn error_page<COMP>(mut self, status: StatusCode) -> Self
    where
        COMP: PageComponent,
        COMP::Properties: DeserializeOwned,
    {
        #[cfg(not(feature = "client"))]
        {
            use futures::TryFutureExt;
            use crate::app::RenderContext;

            self.server_error_router.insert(
                status,
                ErrorPageHandler::new(move |ctx, _status| {
                    let head = super::page_head::PageHead::new();
                    let render_layout = ctx.app_data::<RenderLayout>().cloned().unwrap();
                    let render_ctx = RenderContext::new(ctx, head, render_layout);

                    // Returns the future
                    COMP::render::<BASE>(render_ctx).map_ok(|x| x.into_response())
                }),
            );
        }

        self.add_error_component::<COMP>(status);
        self
    }

    /// Register a page to handle any error.
    #[cfg_attr(feature="client", allow(unused_variables))]
    pub fn error_page_fallback<COMP>(mut self) -> Self
    where
        COMP: PageComponent,
        COMP::Properties: DeserializeOwned
    {
        #[cfg(not(feature = "client"))]
        {
            use futures::TryFutureExt;
            use crate::app::RenderContext;

            self.server_error_router
                .fallback(ErrorPageHandler(Box::new(move |ctx, _status| {
                    let head = super::page_head::PageHead::new();
                    let render_layout = ctx.app_data::<RenderLayout>().cloned().unwrap();
                    let render_ctx = RenderContext::new(ctx, head, render_layout);
                    let res = COMP::render::<BASE>(render_ctx).map_ok(|x| x.into_response());

                    // Returns the future
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
        self.error_page::<NotFoundPage>(StatusCode::NOT_FOUND)
            .error_page_fallback::<ErrorPage>()
    }

    /// Register a server action.
    pub fn action<A>(self) -> Self where A: Action {
        #[cfg(not(feature = "client"))]
        {
            use crate::web::{Body, IntoJsonResponse};

            let route = A::route().to_string();
            let method = A::method();

            self.route(Route::new(&route, method, |ctx: RequestContext| async move {
                let output = crate::try_response!(A::call(ctx).await);
                let json_res = crate::try_response!(output.into_json_response());
                let (parts, body) = json_res.into_parts();
                let bytes = crate::try_response!(serde_json::to_vec(&body));
                let body = Body::from(bytes);
                Response::from_parts(parts, body)
            }))
        }

        #[cfg(feature = "client")]
        self
    }

    /// Adds a shared state that will be shared between server and client.
    pub fn app_data<T>(mut self, data: T) -> Self where T: Send + Sync + 'static {
        self.app_data.insert::<T>(data);
        self
    }

    /// Adds a shared state that will be available on the server.
    #[cfg(not(target_arch="wasm32"))]
    pub fn server_data<T>(self, data: T) -> Self where T: Send + Sync + 'static {
        self.app_data(data)
    }

    /// Adds a shared state that will be available on the server.
    #[cfg(target_arch="wasm32")]
    pub fn server_data<T>(self, _: T) -> Self where T: Send + Sync + 'static {
        self
    }

    /// Adds a shared state that will be available on the client.
    #[cfg(target_arch="wasm32")]
    pub fn client_data<T>(self, data: T) -> Self where T: Send + Sync + 'static {
        self.app_data(data)
    }

    /// Adds a shared state that will be available on the client.
    #[cfg(not(target_arch="wasm32"))]
    pub fn client_data<T>(self, _: T) -> Self where T: Send + Sync + 'static {
        self
    }

    /// Adds headers to always append in a response.
    #[cfg_attr(feature = "client", allow(unused_mut, unused_variables))]
    pub fn default_headers(mut self, headers: DefaultHeaders) -> Self {
        #[cfg(not(feature = "client"))]
        {
            self.default_headers.extend(headers.into_inner());
        }
        self
    }

    /// Adds the given `Hooks`.
    #[cfg(feature = "hooks")]
    pub fn hooks(mut self, hooks: crate::events::Hooks) -> Self {
        self.hooks.extend(hooks);
        self
    }

    /// Constructs an `AppService` using this instance.
    #[allow(clippy::let_and_return)]
    pub fn build(self) -> AppService where BASE: BaseComponent<Properties =ChildrenProps>{
        let App {
            layout,
            server_router,
            page_router: client_router,
            client_error_router,
            server_error_router,
            default_headers,
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

        #[cfg(feature = "hooks")]
        let hooks = Arc::new(hooks);

        // Add startup app data
        app_data.insert::<RenderLayout>(layout); // The RenderContext require the RenderLayout

        #[cfg(feature = "hooks")]
        app_data.insert(hooks.clone());

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
            default_headers,

            #[cfg(feature = "hooks")]
            hooks,
        };

        let service = AppService::new(Arc::new(inner));

        // Initialize
        #[cfg(feature = "hooks")]
        {
            use crate::events::Hooks;

            let hooks = service
                .app_data()
                .get::<Arc<Hooks>>()
                .expect("hooks were not set");

            // FIXME: We only use the initialize hooks once, so must be dropped somehow after being called
            for init in hooks.on_server_initialize_hooks.iter() {
                init.call(service.clone());
            }
        }

        service
    }

    fn add_component<COMP>(&mut self, path: &str)
    where
        COMP: PageComponent,
        COMP::Properties: DeserializeOwned,
    {
        use crate::components::AnyComponent;
        
        log::debug!("Registering component `{}` on {path}", std::any::type_name::<COMP>());

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
pub fn redirection(from: &str, to: &str, status: StatusCode) -> Route {
    let to = to.to_owned();
    Route::any(from, move || {
        let to = to.clone();
        async move {       
            Redirect::new(to, status).expect("invalid redirection")
        }
    })
}