use std::net::SocketAddr;

use hashira::{adapter::Adapter, app::AppService, error::Error};
use rocket::{Build, Rocket};

pub mod core;

pub struct HashiraRocket(Rocket<Build>);

impl HashiraRocket {
    pub fn new() -> HashiraRocket {
        HashiraRocket(Rocket::build())
    }
}

impl From<Rocket<Build>> for HashiraRocket {
    fn from(value: Rocket<Build>) -> Self {
        HashiraRocket(value)
    }
}

#[hashira::async_trait]
impl Adapter for HashiraRocket {
    /// Starts the server.
    async fn serve(self, app: AppService) -> Result<(), Error> {
        let host = hashira::env::get_host().unwrap_or_else(|| String::from("127.0.0.1"));
        let port = hashira::env::get_port().unwrap_or(5000);
        let addr: SocketAddr = format!("{host}:{port}").as_str().parse().unwrap();

        let shutdown = rocket::config::Shutdown {
            ctrlc: false,
            ..rocket::config::Shutdown::default()
        };

        // Attach the router to the rocket
        let rocket = {
            let configure = core::router(app);
            configure(self.0)
        };

        let figment = rocket
            .figment()
            .clone()
            .merge((rocket::Config::ADDRESS, addr.ip()))
            .merge((rocket::Config::PORT, addr.port()))
            //.merge((rocket::Config::LOG_LEVEL, rocket::config::LogLevel::Off))
            .merge((rocket::Config::SHUTDOWN, shutdown));

        println!("⚡ Server started at: `http://{addr}`");
        Rocket::build().configure(figment).launch().await?;
        Ok(())
    }
}
