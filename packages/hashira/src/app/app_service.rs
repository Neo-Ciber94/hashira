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
use std::{rc::Rc, sync::Arc};

pub(crate) struct AppServiceInner<C> {
    pub(crate) layout: RenderLayout<C>,
    pub(crate) server_router: Router<Route<C>>,
    pub(crate) client_router: ClientRouter,
    pub(crate) server_error_router: ServerErrorRouter<C>,
    pub(crate) client_error_router: Arc<ErrorRouter>,
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
    ) -> RequestContext<C> {
        let layout = self.0.layout.clone();
        let client_router = self.0.client_router.clone();
        let client_error_router = self.0.client_error_router.clone();

        RequestContext::new(
            Some(request),
            client_router,
            client_error_router,
            error,
            path,
            layout,
            params,
        )
    }

    /// Returns the server router.
    pub fn server_router(&self) -> &Router<Route<C>> {
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
    pub async fn handle(&self, req: Request, path: &str) -> Response {
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

    /// Returns the `html` template of the layout
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get_layout_html(&self) -> String {
        use crate::server::render_to_static_html;

        let path = String::new(); // TODO: Use Option<String> instead?
        let layout = self.0.layout.clone();
        let client_router = self.0.client_router.clone();
        let client_error_router = self.0.client_error_router.clone();
        let params = Params::new();
        let ctx = RequestContext::new(
            None,
            client_router,
            client_error_router,
            None,
            path,
            layout,
            params,
        );
        let render_layout = &self.0.layout;
        let layout_html = render_layout(ctx).await;
        let html_string = render_to_static_html(move || layout_html).await;
        html_string
    }
}

impl<C> Clone for AppService<C> {
    fn clone(&self) -> Self {
        AppService(self.0.clone())
    }
}
