pub use with_tailwind_web::App;

#[cfg(not(feature = "client"))]
mod server;

// Starts the server
#[cfg(not(feature = "client"))]
#[actix_web::main]
async fn main() -> Result<(), hashira::error::Error> {
    crate::server::start_server::<App>().await
}

// Starts client
#[cfg(feature = "client")]
fn main() {
    // Here any additional logic to run on the client
}
