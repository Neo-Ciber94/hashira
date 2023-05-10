use core::fmt;
use http::Extensions;
use std::future::Future;

use super::RouteMethod;
use crate::{
    app::{Handler, PageHandler},
    web::{FromRequest, IntoResponse},
};

/// Type of the handler in a route
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HandlerKind {
    /// The route is an action
    Action,

    /// The route renders a component page.
    /// If a page return an error an error page will be rendered.
    Page,
}

/// Represents a route for a web server request, including the path, HTTP method,
/// and handler function for the request.
pub struct Route {
    /// The path that the route matches, e.g. "/users/:id" or "/login".
    path: String,

    /// The HTTP method that the route matches, e.g. HttpMethod::GET or HttpMethod::POST.
    method: RouteMethod,

    /// The handler function that should be called when this route matches a request.
    handler: PageHandler,

    /// Route metadata
    extensions: Extensions,
}

impl Route {
    /// Creates a new `ServerPageRoute` with the given path, HTTP method, and handler function.
    pub fn new<H, Args>(path: &str, method: RouteMethod, handler: H) -> Self
    where
        Args: FromRequest + Send + 'static,
        H: Handler<Args> + Sync + Send,
        H::Future: Future + Send + 'static,
        H::Output: IntoResponse,
        <Args as FromRequest>::Fut: Send,
    {
        Route {
            path: path.to_owned(),
            handler: PageHandler::new(handler),
            method,
            extensions: Default::default(),
        }
    }

    /// Returns this route with a new path.
    pub fn with_path(self, new_path: impl Into<String>) -> Self {
        Route {
            path: new_path.into(),
            handler: self.handler,
            method: self.method,
            extensions: self.extensions,
        }
    }

    /// Creates a new `Route` that matches any http method.
    pub fn any<H, Args>(path: &str, handler: H) -> Self
    where
        Args: FromRequest + Send + 'static,
        H: Handler<Args> + Sync + Send,
        H::Future: Future + Send + 'static,
        H::Output: IntoResponse,
        <Args as FromRequest>::Fut: Send,
    {
        Self::new(path, RouteMethod::all(), handler)
    }

    /// Creates a new `Route` with the HTTP method set to POST.
    pub fn post<H, Args>(path: &str, handler: H) -> Self
    where
        Args: FromRequest + Send + 'static,
        H: Handler<Args> + Sync + Send,
        H::Future: Future + Send + 'static,
        H::Output: IntoResponse,
        <Args as FromRequest>::Fut: Send,
    {
        Self::new(path, RouteMethod::POST, handler)
    }

    /// Creates a new `Route` with the HTTP method set to GET.
    pub fn get<H, Args>(path: &str, handler: H) -> Self
    where
        Args: FromRequest + Send + 'static,
        H: Handler<Args> + Sync + Send,
        H::Future: Future + Send + 'static,
        H::Output: IntoResponse,
        <Args as FromRequest>::Fut: Send,
    {
        Self::new(path, RouteMethod::GET, handler)
    }

    /// Creates a new `Route` with the HTTP method set to HEAD.
    pub fn head<H, Args>(path: &str, handler: H) -> Self
    where
        Args: FromRequest + Send + 'static,
        H: Handler<Args> + Sync + Send,
        H::Future: Future + Send + 'static,
        H::Output: IntoResponse,
        <Args as FromRequest>::Fut: Send,
    {
        Self::new(path, RouteMethod::HEAD, handler)
    }

    /// Creates a new `Route` with the HTTP method set to PUT.
    pub fn put<H, Args>(path: &str, handler: H) -> Self
    where
        Args: FromRequest + Send + 'static,
        H: Handler<Args> + Sync + Send,
        H::Future: Future + Send + 'static,
        H::Output: IntoResponse,
        <Args as FromRequest>::Fut: Send,
    {
        Self::new(path, RouteMethod::PUT, handler)
    }

    /// Creates a new `Route` with the HTTP method set to DELETE.
    pub fn delete<H, Args>(path: &str, handler: H) -> Self
    where
        Args: FromRequest + Send + 'static,
        H: Handler<Args> + Sync + Send,
        H::Future: Future + Send + 'static,
        H::Output: IntoResponse,
        <Args as FromRequest>::Fut: Send,
    {
        Self::new(path, RouteMethod::DELETE, handler)
    }

    /// Creates a new `Route` with the HTTP method set to OPTIONS.
    pub fn options<H, Args>(path: &str, handler: H) -> Self
    where
        Args: FromRequest + Send + 'static,
        H: Handler<Args> + Sync + Send,
        H::Future: Future + Send + 'static,
        H::Output: IntoResponse,
        <Args as FromRequest>::Fut: Send,
    {
        Self::new(path, RouteMethod::OPTIONS, handler)
    }

    /// Creates a new `Route` with the HTTP method set to PATCH.
    pub fn patch<H, Args>(path: &str, handler: H) -> Self
    where
        Args: FromRequest + Send + 'static,
        H: Handler<Args> + Sync + Send,
        H::Future: Future + Send + 'static,
        H::Output: IntoResponse,
        <Args as FromRequest>::Fut: Send,
    {
        Self::new(path, RouteMethod::PATCH, handler)
    }

    /// Returns a reference to the path for this `Route`.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the HTTP method for this `Route`.
    pub fn method(&self) -> RouteMethod {
        self.method
    }

    /// Returns a reference to the handler function for this `Route`.
    pub fn handler(&self) -> &PageHandler {
        &self.handler
    }

    /// Metadata of the route.
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    /// A mutable reference to the metadata of the route.
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route")
            .field("path", &self.path)
            .field("method", &self.method)
            .finish()
    }
}
