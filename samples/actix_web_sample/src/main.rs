mod app;
use app::App;

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        #[cfg(feature = "build")]
        mod index;

        #[cfg(not(feature = "build"))]
        mod server;

        // Generate index.html
        #[cfg(feature = "build")]
        #[actix_web::main]
        async fn main() {
            crate::index::generate_html_index::<App>().await;
        }

        // Starts the server
        #[cfg(not(feature = "build"))]
        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            crate::server::start_server::<App>().await
        }

} else {
        mod client;

        // Starts the client
        fn main() {
            crate::client::start_client::<App>();
        }
    }
}
