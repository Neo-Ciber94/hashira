#[cfg(not(target_arch = "wasm32"))]
mod server;

// Starts the server
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    crate::server::start_server().await
}

// Starts client
#[cfg(target_arch = "wasm32")]
fn main() {
    // Here any additional logic to run on the client
}
