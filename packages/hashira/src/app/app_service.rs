use super::{
    error_router::{ErrorRouter, ServerErrorRouter},
    router::PageRouterWrapper,
    RenderLayout, RequestContext, Route,
};
use crate::{
    error::ResponseError,
    web::{Body, IntoResponse, Request, Response, ResponseExt},
};
use http::StatusCode;
use route_recognizer::{Params, Router};
use std::{rc::Rc, sync::Arc};

pub(crate) struct AppServiceInner {
    pub(crate) layout: RenderLayout,
    pub(crate) server_router: Router<Route>,
    pub(crate) client_router: PageRouterWrapper,
    pub(crate) server_error_router: ServerErrorRouter,
    pub(crate) client_error_router: Arc<ErrorRouter>,
}

enum ErrorSource {
    Response(Response),
    Error(ResponseError),
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
            request,
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
                    return Response::with_status(StatusCode::METHOD_NOT_ALLOWED, Body::default());
                }

                let params = mtch.params().clone();
                let ctx = self.create_context(path.to_owned(), req.clone(), params, None);

                let res = route.handler().call(ctx).await;
                let status = res.status();
                if status.is_client_error() || status.is_server_error() {
                    return self
                        .handle_error(path, req, ErrorSource::Response(res))
                        .await;
                }

                res
            }
            Err(_) => {
                let src = ErrorSource::Error(ResponseError::from_status(StatusCode::NOT_FOUND));
                self.handle_error(path, req, src).await
            }
        }
    }

    async fn handle_error(&self, path: &str, req: Arc<Request>, src: ErrorSource) -> Response {
        let err = match src {
            ErrorSource::Response(res) => {
                let status = res.status();

                // We get the message from the error which may be attached to the response
                let message = res
                    .extensions()
                    .get::<ResponseError>()
                    .and_then(|e| e.message())
                    .map(|s| s.to_owned());
                ResponseError::from((status, message))
            }
            ErrorSource::Error(res) => res,
        };

        let status = err.status();
        match self.0.server_error_router.recognize_error(&status) {
            Some(error_handler) => {
                let params = Params::new();
                let ctx = self.create_context(path.to_owned(), req, params, Some(err));

                match error_handler.call(ctx, status).await {
                    Ok(res) => res,
                    Err(err) => match err.downcast::<ResponseError>() {
                        Ok(err) => (*err).into_response(),
                        Err(err) => {
                            (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
                        }
                    },
                }
            }
            None => err.into_response(),
        }
    }
}

impl Clone for AppService {
    fn clone(&self) -> Self {
        AppService(self.0.clone())
    }
}
