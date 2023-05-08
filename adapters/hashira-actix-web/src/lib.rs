pub mod core;

use std::net::SocketAddr;
use actix_web::{web::ServiceConfig, HttpServer};
use hashira::{adapter::Adapter, app::AppService};

/// An adapter for `actix-web`.
#[derive(Clone)]
pub struct HashiraActixWeb<F = ()>(F);

impl HashiraActixWeb<()> {
    /// Constructs an adapter without any configuration.
    pub fn new() -> HashiraActixWeb<()> {
        HashiraActixWeb(())
    }
}

impl Default for HashiraActixWeb<()> {
    fn default() -> Self {
        Self::new()
    }
}

#[hashira::async_trait]
impl<F> Adapter for HashiraActixWeb<F>
where
    F: sealed::ConfigureActixService + Send + Clone + 'static,
{
    /// Starts the server.
    async fn serve(self, app: AppService) -> Result<(), hashira::error::Error> {
        let host = hashira::env::get_host().unwrap_or_else(|| String::from("127.0.0.1"));
        let port = hashira::env::get_port().unwrap_or(5000);
        let addr: SocketAddr = format!("{host}:{port}").as_str().parse().unwrap();

        println!("Server started at: http://{addr}");

        // Create and run the server
        let server = HttpServer::new(move || {
            let config = self.0.clone();
            actix_web::App::new()
                .configure(move |cfg| config.configure(cfg))
                .configure(core::router(app.clone()))
        })
        .bind(addr)?
        .run();

        // Awaits
        server.await?;

        Ok(())
    }
}

impl<F> From<F> for HashiraActixWeb<F>
where
    F: sealed::ConfigureActixService + Send + Clone + 'static,
{
    fn from(value: F) -> Self {
        HashiraActixWeb(value)
    }
}

impl<F> sealed::ConfigureActixService for F
where
    F: FnOnce(&mut ServiceConfig) + Send + Clone + 'static,
{
    fn configure(self, config: &mut ServiceConfig) {
        (self)(config)
    }
}

impl sealed::ConfigureActixService for () {
    fn configure(self, _: &mut ServiceConfig) {}
}

#[doc(hidden)]
pub(crate) mod sealed {
    use actix_web::web::ServiceConfig;

    pub trait ConfigureActixService {
        fn configure(self, config: &mut ServiceConfig);
    }
}
