use super::{error_router::ErrorRouter, router::PageRouterWrapper, AppData};
pub use crate::error::ResponseError;
use crate::{routing::Params, web::Request};
use std::sync::Arc;

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(crate) struct RequestContextInner {
    path: String,
    params: Params,
    app_data: Arc<AppData>,
    pub(crate) client_router: PageRouterWrapper,
    pub(crate) error_router: Arc<ErrorRouter>,
    pub(crate) request: Arc<Request>,
    pub(crate) error: Option<ResponseError>,
}

// FIXME: We could let this type to be cloneable after we move the `render_layout` from here

/// Contains information about the current request.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub struct RequestContext {
    pub(crate) inner: Arc<RequestContextInner>,
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
impl RequestContext {
    pub fn new(
        request: Arc<Request>,
        app_data: Arc<AppData>,
        client_router: PageRouterWrapper,
        error_router: Arc<ErrorRouter>,
        error: Option<ResponseError>,
        path: String,
        params: Params,
    ) -> Self {
        let inner = RequestContextInner {
            path,
            params,
            error,
            app_data,
            request,
            client_router,
            error_router,
        };

        RequestContext {
            inner: Arc::new(inner),
        }
    }
}

impl RequestContext {
    /// Returns the path of the current request.
    pub fn path(&self) -> &str {
        self.inner.path.as_str()
    }

    /// Returns the current request.
    pub fn request(&self) -> &Request {
        self.inner.request.as_ref()
    }

    /// Returns the matching params of the route.
    pub fn params(&self) -> &Params {
        &self.inner.params
    }

    /// Returns the the data for the given type.
    pub fn app_data<T>(&self) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.inner.app_data.get::<T>()
    }
}

// Required to use the `RequestContext` in a context
impl PartialEq for RequestContext {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

// A helper method to prevent cloning the context directly
pub(crate) fn clone_request_context(ctx: &RequestContext) -> RequestContext {
    RequestContext {
        inner: ctx.inner.clone(),
    }
}
