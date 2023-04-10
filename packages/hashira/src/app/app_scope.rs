use super::{ClientPageRoute, Route};
use route_recognizer::Router;
use std::marker::PhantomData;

pub struct AppScope<C> {
    server_router: Router<Route>,
    client_router: Router<ClientPageRoute>,
    _marker: PhantomData<C>,
}

impl<C> AppScope<C> {
    pub fn new() -> Self {
        todo!()
    }
}
