use super::{
    error_router::{ErrorRouter, ServerErrorRouter},
    router::PageRouterWrapper,
    AppData, RequestContext, Route,
};
use crate::{
    error::ResponseError,
    routing::{Params, PathRouter},
    web::{Body, IntoResponse, Request, Response, ResponseExt},
};
use http::{HeaderMap, StatusCode};
use std::sync::Arc;

pub(crate) struct AppServiceInner {
    pub(crate) server_router: PathRouter<Route>,
    pub(crate) client_router: PageRouterWrapper,
    pub(crate) server_error_router: ServerErrorRouter,
    pub(crate) client_error_router: Arc<ErrorRouter>,
    pub(crate) default_headers: HeaderMap,
    pub(crate) app_data: Arc<AppData>,

    #[cfg(feature = "hooks")]
    pub(crate) hooks: Arc<crate::events::Hooks>,
}

enum ErrorSource {
    Response(Response),
    Error(ResponseError),
}

/// The root service used for handling the `hashira` application.
pub struct AppService(Arc<AppServiceInner>);

impl AppService {
    pub(crate) fn new(inner: Arc<AppServiceInner>) -> Self {
        Self(inner)
    }

    /// Create a context to be used in the request.
    pub fn create_context(
        &self,
        request: Arc<Request>,
        params: Params,
        error: Option<ResponseError>,
    ) -> RequestContext {
        let client_router = self.0.client_router.clone();
        let error_router = self.0.client_error_router.clone();
        let app_data = self.0.app_data.clone();

        RequestContext::new(
            request,
            app_data,
            client_router,
            error_router,
            error,
            params,
        )
    }

    /// Returns the server router.
    pub fn server_router(&self) -> &PathRouter<Route> {
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
    pub async fn handle(&self, req: Request) -> Response {
        let mut res = self._handle(req).await;

        // Merge the response headers with the default headers
        if !self.0.default_headers.is_empty() {
            let mut headers =self.0.default_headers.clone();
            headers.extend(res.headers().clone());
            *res.headers_mut() = headers;
        }

        res
    }

    async fn _handle(&self, req: Request) -> Response {
        let req = Arc::new(req);

        // Handle the request normally
        #[cfg(not(feature = "hooks"))]
        {
            self.handle_request(req).await
        }

        #[cfg(feature = "hooks")]
        {
            use crate::{events::Next, types::BoxFuture};

            let hooks = &self.0.hooks.on_handle_hooks;

            if !hooks.is_empty() {
                return self.handle_request(req).await;
            }

            let this = self.clone();
            let next = Box::new(move |req| {
                Box::pin(async move { this.handle_request(req).await }) as BoxFuture<Response>
            }) as Next;

            let handler = hooks.iter().fold(next, move |cur, next_handler| {
                let next_handler = next_handler.clone_handler();
                Box::new(move |req| Box::pin(async move { next_handler.on_handle(req, cur).await }))
            }) as Next;

            // Handle the request
            handler(req).await
        }
    }

    async fn handle_request(&self, req: Arc<Request>) -> Response {
        // We remove the trailing slash from the path,
        // when adding a path we ensure it cannot end with a slash
        // and should start with a slash

        let mut path = req.uri().path().trim();

        // We trim the trailing slash or should we redirect?
        if path.len() > 1 && path.ends_with('/') {
            path = path.trim_end_matches('/');
        }

        match self.0.server_router.find_match(path) {
            Ok(mtch) => {
                let route = mtch.value;

                // Check if the methods matches
                if let Some(m) = route.method() {
                    let method = req.method().into();
                    if !m.matches(&method) {
                        return Response::with_status(
                            StatusCode::METHOD_NOT_ALLOWED,
                            Body::default(),
                        );
                    }
                }

                let params = mtch.params;
                let ctx = self.create_context(req.clone(), params, None);

                let res = route.handler().call(ctx).await;
                let status = res.status();
                if status.is_client_error() || status.is_server_error() {
                    return self.handle_error(req, ErrorSource::Response(res)).await;
                }

                res
            }
            Err(_) => {
                let src = ErrorSource::Error(ResponseError::from_status(StatusCode::NOT_FOUND));
                self.handle_error(req, src).await
            }
        }
    }

    async fn handle_error(&self, req: Arc<Request>, src: ErrorSource) -> Response {
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
        match self.0.server_error_router.find_match(&status) {
            Some(error_handler) => {
                let params = Params::default();
                let ctx = self.create_context(req, params, Some(err));

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
