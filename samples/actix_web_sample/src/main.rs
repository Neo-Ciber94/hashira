mod components;

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        mod index;
        mod server;

        // Server
        #[cfg(not(feature = "build"))]
        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            crate::server::start_server().await
        }

        // Generate index.html
        #[cfg(feature = "build")]
        #[actix_web::main]
        async fn main() {
            crate::index::generate_html_index().await;
        }

} else {
        mod client;

        // Client
        fn main() {
            crate::client::hydrate();
        }
    }
}
