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
        // let host = hashira::env::get_host().unwrap_or_else(|| String::from("127.0.0.1"));
        // let port = hashira::env::get_port().unwrap_or(5000);
        // let addr: SocketAddr = format!("{host}:{port}").as_str().parse().unwrap();

        // println!("⚡ Server started at: `http://{addr}`");

        // let router = self.0.unwrap_or_default().merge(core::router(app));

        // axum::Server::bind(&addr)
        //     .serve(router.into_make_service())
        //     .await?;

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

