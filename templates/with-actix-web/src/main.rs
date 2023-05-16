#[cfg(not(feature = "client"))]
mod server;

// Starts the server
#[cfg(not(feature = "client"))]
#[actix_web::main]
async fn main() -> Result<(), hashira::error::BoxError> {
    crate::server::start_server().await
}

// Starts client
#[cfg(feature = "client")]
fn main() {
    // Here any additional logic to run on the client
}
