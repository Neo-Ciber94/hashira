use super::PageResponse;
use super::{ClientPageRoute, RenderContext, Route};
use crate::components::id::PageId;
use crate::components::PageComponent;
use crate::error::Error;
use serde::de::DeserializeOwned;
use std::future::Future;
use std::{collections::HashMap, marker::PhantomData};

/// Represents a nested route in a `App`.
#[derive(Default)]
pub struct AppNested<BASE> {
    // Inner server routes
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) server_router: HashMap<String, Route>,

    // Inner page router
    pub(crate) page_router: HashMap<String, ClientPageRoute>,

    //
    _marker: PhantomData<BASE>,
}

impl<BASE> AppNested<BASE> {
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
    #[cfg_attr(target_arch="wasm32", allow(unused_mut, unused_variables))]
    pub fn route(mut self, route: Route) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = route.path().to_owned(); // To please the borrow checker
            self.server_router.insert(path, route);
        }

        self
    }

    /// Adds a page for the given route.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn page<COMP, H, Fut>(mut self, path: &str, handler: H) -> Self
    where
        COMP: PageComponent,
        COMP::Properties: DeserializeOwned,
        H: Fn(RenderContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<PageResponse<COMP, BASE>, Error>> + Send + Sync + 'static,
    {
        use super::page_head::PageHead;
        use crate::app::RenderLayout;

        self.add_component::<COMP>(path);

        self.route(Route::get(path, move |ctx| {
            let head = PageHead::new();
            let render_layout = ctx.app_data::<RenderLayout>().cloned().unwrap();
            let render_ctx = RenderContext::new(ctx, head, render_layout);
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
        H: Fn(RenderContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<PageResponse<COMP, BASE>, Error>> + Send + Sync + 'static,
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
pub fn nested<BASE>() -> AppNested<BASE> {
    AppNested::new()
}
