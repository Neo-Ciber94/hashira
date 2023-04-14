use crate::components::id::PageId;
use super::ClientPageRoute;
use route_recognizer::{Match, Router};
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
    path_to_id: Router<PageId>,
    id_to_page: HashMap<PageId, ClientPageRoute>,
}

impl PageRouter {
    /// Constructs an empty router.
    pub fn new() -> Self {
        PageRouter {
            path_to_id: Router::new(),
            id_to_page: HashMap::new(),
        }
    }

    /// Adds a route for the given path.
    pub fn add(&mut self, path: &str, dest: ClientPageRoute) {
        let id = dest.id().clone();
        self.id_to_page.insert(id.clone(), dest);
        self.path_to_id.add(path, id);
    }

    /// Returns the page that matches the given path.
    pub fn recognize(&self, path: &str) -> Option<Match<&ClientPageRoute>> {
        // FIXME: If we can prevent to make a clone for the params, will be better

        let mtch = self.path_to_id.recognize(path).ok()?;
        let id = mtch.handler();
        let page = self.id_to_page.get(*id)?;
        let params = mtch.params().clone();
        Some(Match::new(page, params))
    }

    /// Returns the component with the given id.
    pub fn recognize_by_id(&self, id: &PageId) -> Option<&ClientPageRoute> {
        self.id_to_page.get(id)
    }
}
