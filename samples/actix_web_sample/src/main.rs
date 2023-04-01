mod components;
use components::App;

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        mod index;
        mod server;

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
        mod client;

        // Client
        fn main() {
            crate::client::start_client::<App>();
        }
    }
}
