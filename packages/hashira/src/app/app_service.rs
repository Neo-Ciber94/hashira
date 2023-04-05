use super::{
    error_router::{ErrorRouter, ServerErrorRouter},
    router::ClientRouter,
    RenderLayout, RequestContext, Route,
};
use crate::{
    error::ResponseError,
    web::{Request, Response, ResponseExt},
};
use http::{status, StatusCode};
use route_recognizer::{Params, Router};
use std::{marker::PhantomData, rc::Rc, sync::Arc};

pub(crate) struct AppServiceInner<C> {
    pub(crate) layout: RenderLayout,
    pub(crate) server_router: Router<Route>,
    pub(crate) client_router: ClientRouter,
    pub(crate) server_error_router: ServerErrorRouter,
    pub(crate) client_error_router: Arc<ErrorRouter>,
    pub(crate) _marker: PhantomData<C>, // FIXME: Remove generic?
}

pub struct AppService<C>(Rc<AppServiceInner<C>>);

impl<C> AppService<C> {
    pub(crate) fn new(inner: Rc<AppServiceInner<C>>) -> Self {
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
        let client_router = self.0.client_router.clone();
        let error_router = self.0.client_error_router.clone();

        RequestContext::new(
            Some(request),
            client_router,
            error_router,
            error,
            path,
            params,
        )
    }

    /// Returns the server router.
    pub fn server_router(&self) -> &Router<Route> {
        &self.0.server_router
    }

    /// Returns the client router.
    pub fn client_router(&self) -> &ClientRouter {
        &self.0.client_router
    }

    /// Returns the router for handling error pages on the client.
    pub fn client_error_router(&self) -> &Arc<ErrorRouter> {
        &self.0.client_error_router
    }

    /// Process the incoming request and return the response.
    pub async fn handle(&self, mut req: Request, path: &str) -> Response {
        // Insert request extensions
        let render_layout = self.0.layout.clone();
        req.extensions_mut().insert(render_layout);

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

impl<C> Clone for AppService<C> {
    fn clone(&self) -> Self {
        AppService(self.0.clone())
    }
}
