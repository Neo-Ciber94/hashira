use crate::{
    server::render_to_static_html,
    web::{Request, Response},
};
use route_recognizer::{Params, Router};

use std::rc::Rc;

use super::{AppContext, ClientPageRoute, RenderLayout, ServerPageRoute};

pub(crate) struct Inner<C> {
    pub(crate) layout: RenderLayout<C>,
    pub(crate) server_router: Router<ServerPageRoute<C>>,
    pub(crate) client_router: Router<ClientPageRoute>,
}

pub struct AppService<C>(Rc<Inner<C>>);

impl<C> AppService<C> {
    pub(crate) fn new(inner: Rc<Inner<C>>) -> Self {
        Self(inner)
    }

    /// Create a context to be used in the request.
    pub fn create_context(&self, request: Request, params: Params) -> AppContext<C> {
        let layout = self.0.layout.clone();
        AppContext::new(request, layout, params)
    }

    /// Returns the server router.
    pub fn server_router(&self) -> &Router<ServerPageRoute<C>> {
        &self.0.server_router
    }

    /// Returns the client router.
    pub fn client_router(&self) -> &Router<ClientPageRoute> {
        &self.0.client_router
    }

    /// Process the incoming request and return the response.
    pub async fn handle(&self, req: Request, path: &str) -> Response {
        match self.0.server_router.recognize(&path) {
            Ok(page) => {
                let route = page.handler();
                let params = page.params().clone();
                let ctx = self.create_context(req, params);
                let res = route.handler().call(ctx).await;
                res
            }
            Err(_) => {
                todo!("Return a 404 component")
            }
        }
    }

    /// Returns the `html` template of the layout
    pub async fn get_layout_html(&self) -> String {
        let layout = self.0.layout.clone();
        let params = Params::new();
        let ctx = AppContext::no_request(layout, params);
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
