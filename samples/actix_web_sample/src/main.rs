mod components;
use components::App;

#[cfg(not(target_arch = "wasm32"))]
mod index;

#[cfg(not(target_arch = "wasm32"))]
mod server;

#[cfg(target_arch = "wasm32")]
mod client;

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        // Generate index.html
        #[cfg(feature = "build")]
        #[actix_web::main]
        async fn main() {
            crate::index::generate_html_index::<App>().await;
        }

        // Server
        #[cfg(not(feature = "build"))]
        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            crate::server::start_server::<App>().await
        }

} else {
        // Client
        fn main() {
            crate::client::start_client::<App>();
        }
    }
}
