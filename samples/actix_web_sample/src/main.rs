mod components;

#[cfg(target_arch = "wasm32")]
mod client;

#[cfg(not(target_arch = "wasm32"))]
mod index;

#[cfg(not(target_arch = "wasm32"))]
mod server;

// Client
#[cfg(target_arch = "wasm32")]
fn main() {
    crate::client::hydrate();
}

// Server
#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(feature = "build"))]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    crate::server::start_server().await
}

// Generate index.html 
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "build")]
#[actix_web::main]
async fn main() {
    crate::index::generate_html_index().await;
}
