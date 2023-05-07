#[cfg(not(feature = "client"))]
mod server;

// Starts the server
#[cfg(not(feature = "client"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    crate::server::start_server().await
}

// Starts client
#[cfg(feature = "client")]
fn main() {
    // Here any additional logic to run on the client
}

