use super::{
    error_router::{ErrorRouter, ServerErrorRouter},
    router::PageRouterWrapper,
    RenderLayout, RequestContext, Route,
};
use crate::{
    error::ResponseError,
    web::{Request, Response, ResponseExt},
};
use http::{status, StatusCode};
use route_recognizer::{Params, Router};
use std::{rc::Rc, sync::Arc};

pub(crate) struct AppServiceInner {
    pub(crate) layout: RenderLayout,
    pub(crate) server_router: Router<Route>,
    pub(crate) client_router: PageRouterWrapper,
    pub(crate) server_error_router: ServerErrorRouter,
    pub(crate) client_error_router: Arc<ErrorRouter>,
}

/// The root service used for handling the `hashira` application.
pub struct AppService(Rc<AppServiceInner>);

impl AppService {
    pub(crate) fn new(inner: Rc<AppServiceInner>) -> Self {
        Self(inner)
    }

    /// Create a context to be used in the request.
    pub fn create_context(
        &self,
        path: String,
        request: Arc<Request>,
        params: Params,
        error: Option<ResponseError>,
    ) -> RequestContext {
        let render_layout = self.0.layout.clone();
        let client_router = self.0.client_router.clone();
        let error_router = self.0.client_error_router.clone();

        RequestContext::new(
            Some(request),
            client_router,
            error_router,
            error,
            render_layout,
            path,
            params,
        )
    }

    /// Returns the server router.
    pub fn server_router(&self) -> &Router<Route> {
        &self.0.server_router
    }

    /// Returns the page router.
    pub fn page_router(&self) -> &PageRouterWrapper {
        &self.0.client_router
    }

    /// Returns the router for handling error pages on the client.
    pub fn error_router(&self) -> &Arc<ErrorRouter> {
        &self.0.client_error_router
    }

    /// Process the incoming request and return the response.
    pub async fn handle(&self, req: Request, mut path: &str) -> Response {
        // We remove the trailing slash from the path,
        // when adding a path we ensure it cannot end with a slash
        // and should start with a slash

        path = path.trim();

        // FIXME: Ensure the path always starts with `/`
        debug_assert!(path.starts_with("/"));

        if path.len() > 1 && path.ends_with("/") {
            path = path.trim_end_matches("/");
        }

        // Request now is read-only
        let req = Arc::new(req);

        match self.0.server_router.recognize(&path) {
            Ok(mtch) => {
                let route = mtch.handler();
                let method = req.method().into();

                if !route.method().matches(&method) {
                    return Response::with_status(StatusCode::METHOD_NOT_ALLOWED);
                }

                let params = mtch.params().clone();
                let ctx = self.create_context(path.to_owned(), req.clone(), params, None);

                match route.handler().call(ctx).await {
                    Ok(res) => res,
                    Err(err) => {
                        self.handle_error(ResponseError::from_error(err), path, req)
                            .await
                    }
                }
            }
            Err(_) => {
                self.handle_error(ResponseError::from_status(StatusCode::NOT_FOUND), path, req)
                    .await
            }
        }
    }

    async fn handle_error(&self, err: ResponseError, path: &str, req: Arc<Request>) -> Response {
        let status = err.status();

        match self.0.server_error_router.recognize_error(&status) {
            Some(error_handler) => {
                let params = Params::new();
                let ctx = self.create_context(path.to_owned(), req, params, Some(err));

                match error_handler.call(ctx, status).await {
                    Ok(res) => res,
                    Err(err) => match err.downcast::<ResponseError>() {
                        Ok(err) => {
                            let mut res = Response::text(err.to_string());
                            *res.status_mut() = err.status();
                            res
                        }
                        Err(err) => {
                            let mut res = Response::text(err.to_string());
                            *res.status_mut() = status::StatusCode::INTERNAL_SERVER_ERROR;
                            res
                        }
                    },
                }
            }
            None => Response::with_status(status),
        }
    }
}

impl Clone for AppService {
    fn clone(&self) -> Self {
        AppService(self.0.clone())
    }
}
