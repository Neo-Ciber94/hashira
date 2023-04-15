use super::ClientPageRoute;
use crate::components::id::PageId;
use crate::routing::{PathRouter, RouteMatch};
use std::collections::HashMap;
use std::{ops::Deref, sync::Arc};
use yew::Properties;

/// A wrapper around `PageRouter` to allow it to be used as a `Properties`.
#[derive(Properties)]
pub struct PageRouterWrapper {
    inner: Arc<PageRouter>,
}

impl From<PageRouter> for PageRouterWrapper {
    fn from(value: PageRouter) -> Self {
        PageRouterWrapper {
            inner: Arc::new(value),
        }
    }
}

impl Deref for PageRouterWrapper {
    type Target = PageRouter;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl PartialEq for PageRouterWrapper {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Clone for PageRouterWrapper {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Represents a router for the client.
#[derive(Default)]
pub struct PageRouter {
    path_to_id: PathRouter<PageId>,
    id_to_page: HashMap<PageId, ClientPageRoute>,
}

impl PageRouter {
    /// Constructs an empty router.
    pub fn new() -> Self {
        PageRouter {
            path_to_id: PathRouter::new(),
            id_to_page: HashMap::new(),
        }
    }

    /// Adds a client route.
    pub fn insert(&mut self, route: &str, dest: ClientPageRoute) {
        let id = dest.id().clone();
        self.id_to_page.insert(id.clone(), dest);
        self.path_to_id
            .insert(route, id)
            .expect("failed to add route");
    }

    /// Returns the page that matches the given path.
    pub fn find_match<'a>(&'a self, path: &'a str) -> Option<RouteMatch<&ClientPageRoute>> {
        match self.path_to_id.find_match(path) {
            Ok(RouteMatch { value: id, params }) => {
                if let Some(value) = self.id_to_page.get(id) {
                    Some(RouteMatch { value, params })
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// Returns the component with the given id.
    pub fn find_by_id(&self, id: &PageId) -> Option<&ClientPageRoute> {
        self.id_to_page.get(id)
    }
}
