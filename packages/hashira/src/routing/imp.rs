use matchit::Match;

use super::{InsertError, MatchError, Params, RouteMatch};

pub struct BaseRouter<T> {
    inner: matchit::Router<T>,
}

impl<T> BaseRouter<T> {
    pub fn new() -> Self {
        BaseRouter {
            inner: matchit::Router::new(),
        }
    }

    pub fn insert(&mut self, route: impl Into<String>, value: T) -> Result<(), InsertError> {
        self.inner
            .insert(route, value)
            .map_err(|e| InsertError(e.into()))
    }

    pub fn find(&self, path: impl AsRef<str>) -> Result<RouteMatch<&T>, MatchError> {
        self.inner
            .at(path.as_ref())
            .map(|Match { value, params }| RouteMatch {
                value,
                params: Params::from_iter(params.iter()),
            })
            .map_err(|e| match e {
                matchit::MatchError::NotFound => MatchError::NotFound,
                other => MatchError::Other(other.into()),
            })
    }

    pub fn find_mut(
        &mut self,
        path: impl AsRef<str>,
    ) -> Result<RouteMatch<&mut T>, MatchError> {
        self.inner
            .at_mut(path.as_ref())
            .map(|Match { value, params }| RouteMatch {
                value,
                params: Params::from_iter(params.iter()),
            })
            .map_err(|e| match e {
                matchit::MatchError::NotFound => MatchError::NotFound,
                other => MatchError::Other(other.into()),
            })
    }
}

impl<T> Default for BaseRouter<T> {
    fn default() -> Self {
        Self::new()
    }
}
