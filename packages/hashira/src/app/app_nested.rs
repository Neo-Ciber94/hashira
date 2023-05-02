use super::{ClientPageRoute, Route};
use crate::components::id::PageId;
use crate::components::PageComponent;
use serde::de::DeserializeOwned;
use std::{collections::HashMap, marker::PhantomData};
use yew::html::ChildrenProps;
use yew::BaseComponent;

/// Represents a nested route in a `App`.
#[derive(Default)]
pub struct AppNested<BASE> {
    // Inner server routes
    #[cfg(not(feature = "client"))]
    pub(crate) server_router: HashMap<String, Route>,

    // Inner page router
    pub(crate) page_router: HashMap<String, ClientPageRoute>,

    //
    _marker: PhantomData<BASE>,
}

impl<BASE> AppNested<BASE>
where
    BASE: BaseComponent<Properties = ChildrenProps> + 'static,
{
    /// Creates a new nested route.
    pub fn new() -> Self {
        AppNested {
            #[cfg(not(feature = "client"))]
            server_router: HashMap::new(),
            page_router: HashMap::new(),
            _marker: PhantomData,
        }
    }

    /// Adds a route handler.
    #[cfg_attr(feature = "client", allow(unused_mut, unused_variables))]
    pub fn route(mut self, route: Route) -> Self {
        #[cfg(not(feature = "client"))]
        {
            let path = route.path().to_owned(); // To please the borrow checker
            self.server_router.insert(path, route);
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
        let route = COMP::route().unwrap_or_else(|| {
            panic!(
                "`{}` is not declaring a route",
                std::any::type_name::<COMP>()
            )
        });

        self.add_component::<COMP>(route);

        #[cfg(not(feature = "client"))]
        {
            use crate::app::{RenderLayout, RenderContext};

            self.route(Route::get(route, move |ctx| {
                let head = super::page_head::PageHead::new();
                let render_layout = ctx.app_data::<RenderLayout>().cloned().unwrap();
                let render_ctx = RenderContext::new(ctx, head, render_layout);

                // Returns the future
                COMP::loader::<BASE>(render_ctx)
            }))
        }

        // In the client we don't add pages, just the component
        #[cfg(feature = "client")]
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
pub fn nested<BASE>() -> AppNested<BASE>
where
    BASE: BaseComponent<Properties = ChildrenProps> + 'static,
{
    AppNested::new()
}
