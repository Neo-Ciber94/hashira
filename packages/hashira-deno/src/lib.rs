pub mod core;
mod js_bindings;
use hashira::{adapter::Adapter, app::AppService};
use std::net::SocketAddr;

/// An adapter for `deno`.
pub struct HashiraDeno;

#[hashira::async_trait]
impl Adapter for HashiraDeno {
    /// Starts the server.
    async fn serve(self, app: AppService) -> Result<(), hashira::error::Error> {
        todo!();

        Ok(())
    }
}

impl HashiraDeno {
    /// Constructs a default hashira adapter.
    pub fn new() -> Self {
        HashiraDeno
    }
}
