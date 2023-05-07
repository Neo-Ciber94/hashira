pub mod core;
use hashira::{adapter::Adapter, app::AppService};
use std::net::SocketAddr;

/// An adapter for `axum`.
#[derive(Default)]
pub struct HashiraAxum(Option<axum::Router>);

#[hashira::async_trait]
impl Adapter for HashiraAxum {
    /// Starts the server.
    async fn serve(self, app: AppService) -> Result<(), hashira::error::Error> {
        let host = hashira::env::get_host().unwrap_or_else(|| String::from("127.0.0.1"));
        let port = hashira::env::get_port().unwrap_or(5000);
        let addr: SocketAddr = format!("{host}:{port}").as_str().parse().unwrap();

        println!("Server started at: http://{addr}");

        let router = self.0.unwrap_or_default().merge(core::router(app));

        axum::Server::bind(&addr)
            .serve(router.into_make_service())
            .await?;

        Ok(())
    }
}

impl HashiraAxum {
    /// Constructs a default hashira adapter.
    pub fn new() -> Self {
        HashiraAxum(None)
    }
}

impl From<axum::Router> for HashiraAxum {
    fn from(value: axum::Router) -> Self {
        HashiraAxum(Some(value))
    }
}
