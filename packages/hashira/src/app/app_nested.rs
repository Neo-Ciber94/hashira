use crate::actions::Action;
use crate::components::id::PageId;
use crate::components::PageComponent;
use crate::routing::{ClientPageRoute, Route};
use serde::de::DeserializeOwned;
use std::{collections::HashMap, marker::PhantomData};
use yew::html::ChildrenProps;
use yew::BaseComponent;

/// Marker to specify a nested route should be inserted at the root of the router,
/// and not as a sub route.
///
/// This is just a workaround for allowing to insert actions in specify path.
#[allow(dead_code)]
pub(crate) struct InsertInRootRoute;

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
        // Pages must had a path defined
        let route = COMP::route().unwrap_or_else(|| {
            panic!(
                "`{}` is not declaring a route",
                std::any::type_name::<COMP>()
            )
        });

        self.add_component::<COMP>(route);

        #[cfg(not(feature = "client"))]
        {
            use crate::app::{RenderContext, RenderLayout, RequestContext};
            use crate::routing::HandlerKind;

            let mut route = Route::get(route, move |ctx: RequestContext| {
                let head = super::page_head::PageHead::new();
                let render_layout = ctx.app_data::<RenderLayout>().cloned().unwrap();
                let render_ctx = RenderContext::new(ctx, head, render_layout);

                // Returns the future
                COMP::render::<BASE>(render_ctx)
            });

            route.extensions_mut().insert(HandlerKind::Page);
            self.route(route)
        }

        // In the client we don't add pages, just the component
        #[cfg(feature = "client")]
        self
    }

    /// Register a server action.
    pub fn action<A>(self) -> Self
    where
        A: Action,
    {
        #[cfg(not(feature = "client"))]
        {
            use crate::app::RequestContext;
            use crate::web::{Body, IntoJsonResponse, Response};
            use crate::routing::HandlerKind;
            
            let route = A::route().to_string();
            let method = A::method();
            let mut route = Route::new(&route, method, |ctx: RequestContext, body: Body| async move {
                let output = crate::try_response!(A::call(ctx, body).await);
                let json_res = crate::try_response!(output.into_json_response());
                let (parts, body) = json_res.into_parts();
                let bytes = crate::try_response!(serde_json::to_vec(&body));
                let body = Body::from(bytes);
                Response::from_parts(parts, body)
            });

            route.extensions_mut().insert(InsertInRootRoute);
            route.extensions_mut().insert(HandlerKind::Action);
            self.route(route)
        }

        #[cfg(feature = "client")]
        self
    }

    fn add_component<COMP>(&mut self, path: &str)
    where
        COMP: PageComponent,
        COMP::Properties: DeserializeOwned,
    {
        use crate::components::AnyComponent;

        log::debug!(
            "Registering component `{}` on path: {path}",
            std::any::type_name::<COMP>()
        );

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
