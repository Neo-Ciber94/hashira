use crate::web::FromRequest;
use std::{
    future::{ready, Ready},
    net::SocketAddr,
    ops::Deref,
};
use thiserror::Error;

/// An error when failed to resolve an address.
#[derive(Debug, Error)]
#[error("unable to retrieve address")]
pub struct AddrNotFoundError;

/// Remote address connecting to the server.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RemoteAddr(SocketAddr);

impl RemoteAddr {
    /// Returns the inner value.
    pub fn into_inner(self) -> SocketAddr {
        self.0
    }
}

impl From<SocketAddr> for RemoteAddr {
    fn from(value: SocketAddr) -> Self {
        RemoteAddr(value)
    }
}

impl Deref for RemoteAddr {
    type Target = SocketAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for RemoteAddr {
    type Error = AddrNotFoundError;
    type Fut = Ready<Result<RemoteAddr, AddrNotFoundError>>;

    fn from_request(ctx: &crate::app::RequestContext, _body: &mut crate::web::Body) -> Self::Fut {
        let addr = ctx
            .request()
            .extensions()
            .get::<RemoteAddr>()
            .cloned()
            .ok_or(AddrNotFoundError);

        ready(addr)
    }
}
