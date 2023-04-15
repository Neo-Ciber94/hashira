use std::future::Future;

use http::Method;

use super::{PageHandler, RequestContext};
use crate::{
    components::{id::PageId, AnyComponent},
    web::IntoResponse,
};

// Represents a client-side page route, containing a component and a path pattern.
#[derive(Clone)]
pub struct ClientPageRoute {
    pub(crate) page_id: PageId,
    pub(crate) component: AnyComponent<serde_json::Value>, // The component for this page route.
    pub(crate) path: String,                               // The path pattern for this page route.
}

impl ClientPageRoute {
    /// Returns the id of the page of this route.
    pub fn id(&self) -> &PageId {
        &self.page_id
    }

    // Renders the component for this page route with the given props.
    pub fn render(&self, props: serde_json::Value) -> yew::Html {
        self.component.render_with_props(props)
    }

    // Returns a reference to the path pattern for this page route.
    pub fn path(&self) -> &str {
        self.path.as_str()
    }
}

/// Represents an HTTP method as a bit field. This is a compact representation
/// of the HTTP method that allows for efficient matching of multiple methods
/// at once.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct HttpMethod(u8);

impl HttpMethod {
    /// The HTTP GET method.
    pub const GET: HttpMethod = HttpMethod(0b0001);

    /// The HTTP POST method.
    pub const POST: HttpMethod = HttpMethod(0b0010);

    /// The HTTP PUT method.
    pub const PUT: HttpMethod = HttpMethod(0b0100);

    /// The HTTP PATCH method.
    pub const PATCH: HttpMethod = HttpMethod(0b1000);

    /// The HTTP DELETE method.
    pub const DELETE: HttpMethod = HttpMethod(0b0001_0000);

    /// The HTTP HEAD method.
    pub const HEAD: HttpMethod = HttpMethod(0b0010_0000);

    /// The HTTP OPTIONS method.
    pub const OPTIONS: HttpMethod = HttpMethod(0b0100_0000);

    /// The HTTP TRACE method.
    pub const TRACE: HttpMethod = HttpMethod(0b1000_0000);

    /// Returns true if this `HttpMethod` matches the given `HttpMethod`.
    ///
    /// Matching is done by bitwise ANDing the bit fields of the two `HttpMethod`s.
    /// If the result is non-zero, the two methods match.
    pub fn matches(&self, other: &HttpMethod) -> bool {
        (self.0 & other.0) != 0
    }
}

impl std::ops::BitOr for HttpMethod {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        HttpMethod(self.0 | other.0)
    }
}

impl From<&Method> for HttpMethod {
    fn from(value: &Method) -> Self {
        match *value {
            Method::GET => HttpMethod::GET,
            Method::POST => HttpMethod::POST,
            Method::PUT => HttpMethod::PUT,
            Method::DELETE => HttpMethod::DELETE,
            Method::HEAD => HttpMethod::HEAD,
            Method::OPTIONS => HttpMethod::OPTIONS,
            Method::PATCH => HttpMethod::PATCH,
            Method::TRACE => HttpMethod::TRACE,
            _ => panic!("unsupported http method: {value}"),
        }
    }
}

impl From<Method> for HttpMethod {
    fn from(value: Method) -> Self {
        HttpMethod::from(&value)
    }
}

/// Represents a route for a web server request, including the path, HTTP method,
/// and handler function for the request.
pub struct Route {
    /// The path that the route matches, e.g. "/users/:id" or "/login".
    path: String,
    /// The HTTP method that the route matches, e.g. HttpMethod::GET or HttpMethod::POST.
    method: HttpMethod,
    /// The handler function that should be called when this route matches a request.
    handler: PageHandler,
}

impl Route {
    /// Creates a new `ServerPageRoute` with the given path, HTTP method, and handler function.
    pub fn new<H, R, Fut>(path: &str, method: HttpMethod, handler: H) -> Self
    where
        H: Fn(RequestContext) -> Fut + Send + Sync + 'static,
        R: IntoResponse,
        Fut: Future<Output = R> + 'static,
    {
        assert_valid_path(path);

        Route {
            path: path.to_owned(),
            handler: PageHandler::new(handler),
            method,
        }
    }

    /// Creates a new `Route` with the HTTP method set to POST.
    pub fn post<H, R, Fut>(path: &str, handler: H) -> Self
    where
        H: Fn(RequestContext) -> Fut + Send + Sync + 'static,
        R: IntoResponse,
        Fut: Future<Output = R> + 'static,
    {
        Self::new(path, HttpMethod::POST, handler)
    }

    /// Creates a new `Route` with the HTTP method set to GET.
    pub fn get<H, R, Fut>(path: &str, handler: H) -> Self
    where
        H: Fn(RequestContext) -> Fut + Send + Sync + 'static,
        R: IntoResponse,
        Fut: Future<Output = R> + 'static,
    {
        Self::new(path, HttpMethod::GET, handler)
    }

    /// Creates a new `Route` with the HTTP method set to HEAD.
    pub fn head<H, R, Fut>(path: &str, handler: H) -> Self
    where
        H: Fn(RequestContext) -> Fut + Send + Sync + 'static,
        R: IntoResponse,
        Fut: Future<Output = R> + 'static,
    {
        Self::new(path, HttpMethod::HEAD, handler)
    }

    /// Creates a new `Route` with the HTTP method set to PUT.
    pub fn put<H, R, Fut>(path: &str, handler: H) -> Self
    where
        H: Fn(RequestContext) -> Fut + Send + Sync + 'static,
        R: IntoResponse,
        Fut: Future<Output = R> + 'static,
    {
        Self::new(path, HttpMethod::PUT, handler)
    }

    /// Creates a new `Route` with the HTTP method set to DELETE.
    pub fn delete<H, R, Fut>(path: &str, handler: H) -> Self
    where
        H: Fn(RequestContext) -> Fut + Send + Sync + 'static,
        R: IntoResponse,
        Fut: Future<Output = R> + 'static,
    {
        Self::new(path, HttpMethod::DELETE, handler)
    }

    /// Creates a new `Route` with the HTTP method set to OPTIONS.
    pub fn options<H, R, Fut>(path: &str, handler: H) -> Self
    where
        H: Fn(RequestContext) -> Fut + Send + Sync + 'static,
        R: IntoResponse,
        Fut: Future<Output = R> + 'static,
    {
        Self::new(path, HttpMethod::OPTIONS, handler)
    }

    /// Creates a new `Route` with the HTTP method set to PATCH.
    pub fn patch<H, R, Fut>(path: &str, handler: H) -> Self
    where
        H: Fn(RequestContext) -> Fut + Send + Sync + 'static,
        R: IntoResponse,
        Fut: Future<Output = R> + 'static,
    {
        Self::new(path, HttpMethod::PATCH, handler)
    }

    /// Returns a reference to the path for this `Route`.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the HTTP method for this `Route`.
    pub fn method(&self) -> HttpMethod {
        self.method
    }

    /// Returns a reference to the handler function for this `Route`.
    pub fn handler(&self) -> &PageHandler {
        &self.handler
    }
}

pub(crate) fn assert_valid_path(path: &str) {
    assert!(!path.is_empty(), "route path cannot be empty");

    assert!(
        !path.starts_with(' ') || !path.ends_with(' '),
        "route path cannot starts or end with a whitespace but was: {path}"
    );

    assert!(
        path.starts_with('/'),
        "route path must start with `/`, but was: {path}"
    );

    if path.len() > 1 {
        assert!(
            !path.ends_with('/'),
            "route path cannot end with `/` but was: {path}"
        );
    }
}
