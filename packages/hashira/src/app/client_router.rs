use super::ClientPageRoute;
use route_recognizer::Router;
use std::{ops::Deref, sync::Arc};
use yew::Properties;

#[derive(Properties)]
pub struct ClientRouter {
    inner: Arc<Router<ClientPageRoute>>,
}

impl From<Router<ClientPageRoute>> for ClientRouter {
    fn from(inner: Router<ClientPageRoute>) -> Self {
        ClientRouter {
            inner: Arc::new(inner),
        }
    }
}

impl Deref for ClientRouter {
    type Target = Router<ClientPageRoute>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl PartialEq for ClientRouter {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Clone for ClientRouter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
