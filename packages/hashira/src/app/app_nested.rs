use super::{ClientPageRoute, RenderContext, Route};
use crate::components::id::PageId;
use crate::components::PageComponent;
use crate::{error::Error, web::Response};
use serde::de::DeserializeOwned;
use std::future::Future;
use std::{collections::HashMap, marker::PhantomData};

/// Represents a nested route in a `App`.
#[derive(Default)]
pub struct AppNested<C> {
    // Inner server routes
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) server_router: HashMap<String, Route>,

    // Inner page router
    pub(crate) page_router: HashMap<String, ClientPageRoute>,

    //
    _marker: PhantomData<C>,
}

impl<C> AppNested<C> {
    /// Creates a new nested route.
    pub fn new() -> Self {
        AppNested {
            #[cfg(not(target_arch = "wasm32"))]
            server_router: HashMap::new(),
            page_router: HashMap::new(),
            _marker: PhantomData,
        }
    }

    /// Adds a route handler.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn route(mut self, route: Route) -> Self {
        let path = route.path().to_owned(); // To please the borrow checker
        self.server_router.insert(path, route);
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
        COMP: PageComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext<COMP, C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, Error>> + 'static,
    {
        use super::page_head::PageHead;

        self.add_component::<COMP>(path);

        self.route(Route::get(path, move |ctx| {
            let head = PageHead::new();
            let render_ctx = RenderContext::new(ctx, head);
            let fut = handler(render_ctx);
            async { fut.await }
        }))
    }

    /// Adds a page for the given route.
    #[cfg(target_arch = "wasm32")]
    pub fn page<COMP, H, Fut>(mut self, path: &str, _: H) -> Self
    where
        COMP: PageComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext<COMP, C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, Error>> + 'static,
    {
        self.add_component::<COMP>(path);
        self
    }

    fn add_component<COMP>(&mut self, path: &str)
    where
        COMP: PageComponent,
        COMP::Properties: DeserializeOwned,
    {
        use crate::components::AnyComponent;

        self.page_router.insert(
            path.to_owned(),
            ClientPageRoute {
                path: path.to_owned(),
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
}

/// Creates a new nested app.
pub fn nested<C>() -> AppNested<C> {
    AppNested::new()
}
