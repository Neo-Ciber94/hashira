use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use crate::utils::{create_app, start_server_in_random_port, ServerHandle};
use hashira::events::Hooks;

#[tokio::test]
async fn on_server_init_test() {
    let counter = Arc::new(AtomicU64::new(0));

    let service = create_app()
        .hooks(Hooks::new().on_server_initialize({
            let counter = counter.clone();
            move |_service| {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        }))
        .build();

    let ServerHandle {
        shutdown,
        host,
        port,
    } = start_server_in_random_port(service).await;

    let res = crate::utils::get(&format!("http://{host}:{port}/home")).await;
    crate::utils::assert_content_type(&res, "text/html");

    assert_eq!(counter.load(Ordering::Acquire), 1);

    shutdown.send(()).unwrap();
}
