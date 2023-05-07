use super::{InsertError, MatchError, RouteMatch};
use crate::{
    error::Error,
    routing::{PathRouter, Route, RouteMethod},
};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct RouteId(usize);

impl RouteId {
    fn next() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = 1 + NEXT_ID.fetch_add(1, Ordering::Relaxed);
        RouteId(id)
    }
}

#[derive(Debug)]
pub(crate) struct MethodToRoute {
    route: Route,
    method: RouteMethod,
}

#[derive(Default)]
pub(crate) struct MethodRouter(Vec<MethodToRoute>);

/// When the route method conflicts with other already added route.
#[derive(Debug, Error)]
#[error("route conflicts with the method: {0:?}")]
pub struct RouteMethodConflict(RouteMethod);

impl MethodRouter {
    pub(crate) fn get(&self, m: RouteMethod) -> Option<&MethodToRoute> {
        self.0.iter().find(|route| route.method.matches(&m))
    }

    pub(crate) fn push(&mut self, route: Route) -> Result<(), RouteMethodConflict> {
        let method = route.method();

        if self.0.is_empty() {
            self.0.push(MethodToRoute { route, method });
            return Ok(());
        }

        let conflict = self.0.iter().find(|route| route.method.matches(&method));

        if conflict.is_some() {
            let method = conflict.map(|m| m.method).unwrap_or(RouteMethod::all());
            return Err(RouteMethodConflict(method));
        }

        self.0.push(MethodToRoute { method, route });
        Ok(())
    }
}

/// An error ocurred while adding a new route.
#[derive(Debug, Error)]
pub enum InsertServerRouteError {
    /// Match error with other route.
    #[error(transparent)]
    Match(MatchError),

    /// Insert error.
    #[error(transparent)]
    Insert(InsertError),

    /// Conflict with other route method.
    #[error(transparent)]
    MethodConflict(RouteMethodConflict),
}

/// An error when matching a route.
#[derive(Debug, Error)]
pub enum ServerRouterMatchError {
    /// The route don't exists.
    #[error("route not found")]
    NotFound,

    /// The path exists but doesn't match any method.
    #[error("method not allowed")]
    MethodMismatch,

    /// Other match error.
    #[error(transparent)]
    Other(Error),
}

#[derive(Default)]
pub struct ServerRouter {
    path_to_id: BTreeMap<String, RouteId>,
    id_to_route: BTreeMap<RouteId, MethodRouter>,
    route_to_id: PathRouter<RouteId>,
}

impl ServerRouter {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn at(
        &self,
        path: &str,
        method: RouteMethod,
    ) -> Result<RouteMatch<&Route>, ServerRouterMatchError> {
        match self.route_to_id.find(path) {
            Ok(mtch) => {
                let id = mtch.value;
                // SAFETY: If the id exists the route also exists
                let method_router = self.id_to_route.get(id).unwrap();
                let route_mtch = method_router
                    .get(method)
                    .ok_or_else(|| ServerRouterMatchError::MethodMismatch)?;

                Ok(RouteMatch {
                    params: mtch.params.clone(),
                    value: &route_mtch.route,
                })
            }
            Err(MatchError::NotFound) => Err(ServerRouterMatchError::NotFound),
            Err(err) => Err(ServerRouterMatchError::Other(err.into())),
        }
    }

    pub fn insert(&mut self, route: Route) -> Result<(), InsertServerRouteError> {
        let path = route.path().to_owned();
        match self.path_to_id.get(&path) {
            Some(id) => {
                // SAFETY: If the id exists the method router also exists
                let method_router = self.id_to_route.get_mut(id).unwrap();
                method_router
                    .push(route)
                    .map_err(InsertServerRouteError::MethodConflict)
            }
            None => {
                
                let id = RouteId::next();
                let mut method_router = MethodRouter::default();
                method_router.push(route).unwrap(); // SAFETY: This is the first route, so no conflicts

                self.path_to_id.insert(path.clone(), id);
                self.id_to_route.insert(id, method_router);
                self.route_to_id
                    .insert(path, id)
                    .map_err(InsertServerRouteError::Insert)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn noop() {}

    #[test]
    fn insert_test() {
        let mut router = ServerRouter::new();

        let route1 = Route::new("/", RouteMethod::GET, noop);
        let route2 = Route::new("/path", RouteMethod::POST, noop);

        assert!(router.insert(route1).is_ok());
        assert!(router.insert(route2).is_ok());
    }

    #[test]
    fn insert_different_methods_test() {
        let mut router = ServerRouter::new();

        let route1 = Route::new("/", RouteMethod::GET, noop);
        let route2 = Route::new("/", RouteMethod::POST, noop);

        assert!(router.insert(route1).is_ok());
        assert!(router.insert(route2).is_ok());
    }

    #[test]
    fn insert_method_conflict_test() {
        let mut router = ServerRouter::new();

        let route1 = Route::new("/", RouteMethod::GET, noop);
        let route2 = Route::new("/", RouteMethod::GET, noop);

        assert!(router.insert(route1).is_ok());
        assert!(router.insert(route2).is_err());
    }

    #[test]
    fn insert_invalid_path_test() {
        let mut router = ServerRouter::new();

        let route1 = Route::new("/hello/", RouteMethod::GET, noop);
        let route2 = Route::new("", RouteMethod::GET, noop);

        assert!(router.insert(route1).is_err());
        assert!(router.insert(route2).is_err());
    }

    #[test]
    fn at_test() {
        let mut router = ServerRouter::new();

        let route1 = Route::new("/", RouteMethod::GET, noop);
        let route2 = Route::new("/path", RouteMethod::POST, noop);

        assert!(router.insert(route1).is_ok());
        assert!(router.insert(route2).is_ok());

        assert!(router.at("/", RouteMethod::GET).is_ok());
        assert!(router.at("/path", RouteMethod::POST).is_ok());

        assert!(router.at("/", RouteMethod::DELETE).is_err());
        assert!(router.at("/path", RouteMethod::GET).is_err());
    }

    #[test]
    fn at_not_found_test() {
        let mut router = ServerRouter::new();

        let route1 = Route::new("/path", RouteMethod::GET, noop);

        assert!(router.insert(route1).is_ok());

        assert!(matches!(
            router.at("/other", RouteMethod::GET),
            Err(ServerRouterMatchError::NotFound)
        ));
    }

    #[test]
    fn at_method_mismatch_test() {
        let mut router = ServerRouter::new();

        let route1 = Route::new("/path", RouteMethod::GET, noop);

        assert!(router.insert(route1).is_ok());

        assert!(matches!(
            router.at("/path", RouteMethod::POST),
            Err(ServerRouterMatchError::MethodMismatch)
        ));
    }

    #[test]
    fn at_any_method_route_test() {
        let mut router = ServerRouter::new();

        let route1 = Route::new("/path", RouteMethod::all(), noop);

        assert!(router.insert(route1).is_ok());

        assert!(router.at("/path", RouteMethod::GET).is_ok());
        assert!(router.at("/path", RouteMethod::POST).is_ok());
        assert!(router.at("/path", RouteMethod::PUT).is_ok());
        assert!(router.at("/path", RouteMethod::PATCH).is_ok());
        assert!(router.at("/path", RouteMethod::DELETE).is_ok());
    }
}
