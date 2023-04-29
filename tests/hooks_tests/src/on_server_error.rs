use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use crate::utils::{create_app, start_server_in_random_port, ServerHandle};
use hashira::events::Hooks;
use reqwest::StatusCode;

#[tokio::test]
async fn on_server_error_test() {
    let counter = Arc::new(AtomicU64::new(0));

    let service = create_app()
        .use_default_error_pages()
        .hooks(Hooks::new().on_server_error({
            let counter = counter.clone();
            move |res| {
                counter.fetch_add(1, Ordering::Relaxed);
                res
            }
        }))
        .build();

    let ServerHandle {
        shutdown,
        host,
        port,
    } = start_server_in_random_port(service).await;

    let res = crate::utils::get(&format!("http://{host}:{port}/this_route_not_exists")).await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert_eq!(counter.load(Ordering::Acquire), 1);
    crate::utils::assert_content_type(&res, "text/html");

    shutdown.send(()).unwrap();
}
