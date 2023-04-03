use super::{
    client_router::ClientRouter,
    error_router::{ClientErrorRouter, ServerErrorRouter},
    AppContext, RenderLayout, ServerPageRoute,
};
use crate::web::{Request, Response, ResponseExt};
use http::StatusCode;
use route_recognizer::{Params, Router};
use std::{rc::Rc, sync::Arc};

pub(crate) struct Inner<C> {
    pub(crate) layout: RenderLayout<C>,
    pub(crate) server_router: Router<ServerPageRoute<C>>,
    pub(crate) client_router: ClientRouter,
    pub(crate) server_error_router: ServerErrorRouter<C>,
    pub(crate) client_error_router: Arc<ClientErrorRouter>,
}

pub struct AppService<C>(Rc<Inner<C>>);

impl<C> AppService<C> {
    pub(crate) fn new(inner: Rc<Inner<C>>) -> Self {
        Self(inner)
    }

    /// Create a context to be used in the request.
    pub fn create_context(&self, path: String, request: Request, params: Params) -> AppContext<C> {
        let layout = self.0.layout.clone();
        let client_router = self.0.client_router.clone();
        let client_error_router = self.0.client_error_router.clone();

        AppContext::new(
            request,
            client_router,
            client_error_router,
            path,
            layout,
            params,
        )
    }

    /// Returns the server router.
    pub fn server_router(&self) -> &Router<ServerPageRoute<C>> {
        &self.0.server_router
    }

    /// Returns the client router.
    pub fn client_router(&self) -> &ClientRouter {
        &self.0.client_router
    }

    /// Returns the router for handling error pages on the client.
    pub fn client_error_router(&self) -> &Arc<ClientErrorRouter> {
        &self.0.client_error_router
    }

    /// Process the incoming request and return the response.
    pub async fn handle(&self, req: Request, path: &str) -> Response {
        match self.0.server_router.recognize(&path) {
            Ok(page) => {
                let route = page.handler();
                let params = page.params().clone();
                let ctx = self.create_context(path.to_owned(), req, params);

                // TODO: Change return type to `Result<Response>` to handle with error pages
                let res = route.handler().call(ctx).await;
                res
            }
            Err(_) => {
                match self
                    .0
                    .server_error_router
                    .recognize_error(&StatusCode::NOT_FOUND)
                {
                    Some(handler) => {
                        let params = Params::new();
                        let ctx = self.create_context(path.to_owned(), req, params);
                        let res = handler.call(ctx, StatusCode::NOT_FOUND).await;
                        res
                    }
                    None => Response::with_status(StatusCode::NOT_FOUND),
                }
            }
        }
    }

    /// Returns the `html` template of the layout
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get_layout_html(&self) -> String {
        use crate::server::render_to_static_html;

        let path = String::new(); // TODO: Use Option<String> instead
        let layout = self.0.layout.clone();
        let client_router = self.0.client_router.clone();
        let client_error_router = self.0.client_error_router.clone();
        let params = Params::new();
        let ctx = AppContext::no_request(client_router, client_error_router, path, layout, params);
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
