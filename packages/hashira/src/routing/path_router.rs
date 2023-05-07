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
        let route = route.into();
        assert_valid_route(&route).map_err(|err| InsertError(err.into()))?;
        self.imp.insert(route, value)
    }

    /// Returns the match for the given path or error if not found.
    pub fn find(&self, path: impl AsRef<str>) -> Result<RouteMatch<&T>, MatchError> {
        self.imp.find(path)
    }

    /// Returns a mutable reference to the match for the given path or error if not found.
    pub fn find_mut(&mut self, path: impl AsRef<str>) -> Result<RouteMatch<&mut T>, MatchError> {
        self.imp.find_mut(path)
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

pub(crate) fn assert_valid_route(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err(String::from("route path cannot be empty"));
    }

    if path.starts_with(' ') || path.ends_with(' ') {
        return Err(format!(
            "route path cannot starts or end with a whitespace but was: {}",
            path
        ));
    }

    if !path.starts_with('/') {
        return Err(format!("route path must start with `/`, but was: {}", path));
    }

    if path.len() > 1 && path.ends_with('/') {
        return Err(format!("route path cannot end with `/` but was: {}", path));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_route() {
        let result = assert_valid_route("/test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_empty_route() {
        let result = assert_valid_route("");
        assert_eq!(
            result.err(),
            Some(String::from("route path cannot be empty"))
        );
    }

    #[test]
    fn test_invalid_whitespace_route() {
        let result = assert_valid_route(" /test");
        assert_eq!(
            result.err(),
            Some(String::from(
                "route path cannot starts or end with a whitespace but was:  /test"
            ))
        );
    }

    #[test]
    fn test_invalid_starting_char_route() {
        let result = assert_valid_route("test");
        assert_eq!(
            result.err(),
            Some(String::from(
                "route path must start with `/`, but was: test"
            ))
        );
    }

    #[test]
    fn test_invalid_ending_char_route() {
        let result = assert_valid_route("/test/");
        assert_eq!(
            result.err(),
            Some(String::from(
                "route path cannot end with `/` but was: /test/"
            ))
        );
    }

    #[test]
    fn test_insert() {
        let mut router = PathRouter::new();
        let route = "/test/:param1/:param2";
        let value = "test-value";

        let result = router.insert(route, value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_match() {
        let mut router = PathRouter::new();
        let route = "/test/:param1/:param2";
        let value = "test-value";
        router.insert(route, value).unwrap();

        let result = router.find("/test/abc/def");
        assert!(result.is_ok());

        let match_result = result.unwrap();
        assert_eq!(
            match_result.params.get("param1"),
            Some(String::from("abc").as_str())
        );
        assert_eq!(
            match_result.params.get("param2"),
            Some(String::from("def").as_str())
        );
        assert_eq!(*match_result.value, "test-value");
    }

    #[test]
    fn test_find_match_not_found() {
        let router = PathRouter::<&str>::new();

        let result = router.find("/not-found");
        assert!(matches!(result.err().unwrap(), MatchError::NotFound));
    }
}
