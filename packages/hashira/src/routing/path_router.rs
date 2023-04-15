use super::{imp, Params};
use thiserror::Error;

/// The result of a match.
pub struct RouteMatch<T> {
    /// The value of the route
    pub value: T,

    /// The resulting params.
    pub params: Params,
}

/// A router.
pub struct PathRouter<T> {
    imp: imp::BaseRouter<T>,
}

impl<T> PathRouter<T> {
    /// Constructs a new router.
    pub fn new() -> Self {
        PathRouter {
            imp: imp::BaseRouter::new(),
        }
    }

    /// Insert the given value at the given route.
    pub fn insert(&mut self, route: impl Into<String>, value: T) -> Result<(), InsertError> {
        self.imp.insert(route, value)
    }

    /// Returns the match for the given path or error if not found.
    pub fn find_match(&self, path: impl AsRef<str>) -> Result<RouteMatch<&T>, MatchError> {
        self.imp.find_match(path)
    }
}

impl<T> Default for PathRouter<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// An error when inserting a route.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct InsertError(pub(crate) Box<dyn std::error::Error + Send + Sync>);

/// An error when is unable to find a route.
#[derive(Debug, Error)]
pub enum MatchError {
    /// The route was not found.
    #[error("route not found")]
    NotFound,

    /// The route was not found due other error.
    #[error(transparent)]
    Other(Box<dyn std::error::Error + Send + Sync>),
}
