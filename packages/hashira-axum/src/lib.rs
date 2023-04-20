pub mod core;
use hashira::app::AppService;
use std::net::SocketAddr;

/// A basic hashira adapter for `axum`.
pub struct HashiraAxum(Option<axum::Router>);

impl HashiraAxum {
    /// Constructs a default hashira adapter.
    pub fn new() -> Self {
        HashiraAxum(None)
    }

    /// Starts the server.
    pub async fn serve(
        self,
        app: AppService,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let host = hashira::env::get_host().unwrap_or_else(|| String::from("127.0.0.1"));
        let port = hashira::env::get_port().unwrap_or(5000);
        let addr: SocketAddr = format!("{host}:{port}").as_str().parse().unwrap();

        println!("⚡ Server started at: `http://{addr}`");

        let router = self.0.unwrap_or_default().merge(core::router(app));

        axum::Server::bind(&addr)
            .serve(router.into_make_service())
            .await?;

        Ok(())
    }
}

impl From<axum::Router> for HashiraAxum {
    fn from(value: axum::Router) -> Self {
        HashiraAxum(Some(value))
    }
}
