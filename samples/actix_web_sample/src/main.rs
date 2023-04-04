mod app;
use app::{hashira, App};

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        #[cfg(not(feature = "build"))]
        mod server;

        // Generate index.html
        #[cfg(feature = "build")]
        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
            
            let service = hashira::<App>();
            service.generate_index_html().await
        }

        // Starts the server
        #[cfg(not(feature = "build"))]
        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            crate::server::start_server::<App>().await
        }

} else {
        // Starts the client
        fn main() {
            wasm_logger::init(wasm_logger::Config::default());
            log::debug!("Hydrating app...");

            let service = hashira::<App>();
            hashira::client::mount_to(service);
        }
    }
}
