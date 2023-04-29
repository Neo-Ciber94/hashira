use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

use crate::utils::{create_app, start_server_in_random_port, ServerHandle};
use hashira::events::{Hooks, Next};

#[tokio::test]
async fn on_handle_hook_test() {
    let count = Arc::new(AtomicU64::new(0));

    let service = create_app()
        .hooks(Hooks::new().on_handle({
            let count = count.clone();
            move |req, next: Next| {
                count.fetch_add(1, Ordering::Relaxed);
                next(req)
            }
        }))
        .build();

    let ServerHandle {
        shutdown,
        host,
        port,
    } = start_server_in_random_port(service).await;

    let res1 = crate::utils::get(&format!("http://{host}:{port}/home")).await;
    crate::utils::assert_content_type(&res1, "text/html");

    let res2 = crate::utils::get(&format!("http://{host}:{port}/home")).await;
    crate::utils::assert_content_type(&res2, "text/html");

    assert_eq!(count.load(Ordering::Acquire), 2);
    shutdown.send(()).unwrap();
}

#[tokio::test]
async fn on_handle_hook_ordered_test() {
    let values = Arc::new(Mutex::new(vec![]));

    let service = create_app()
        .hooks(
            Hooks::new()
                .on_handle({
                    let values = values.clone();
                    move |req, next: Next| {
                        values.lock().unwrap().push(1);
                        next(req)
                    }
                })
                .on_handle({
                    let values = values.clone();
                    move |req, next: Next| {
                        values.lock().unwrap().push(2);
                        next(req)
                    }
                })
                .on_handle({
                    let values = values.clone();
                    move |req, next: Next| {
                        values.lock().unwrap().push(3);
                        next(req)
                    }
                }),
        )
        .build();

    let ServerHandle {
        shutdown,
        host,
        port,
    } = start_server_in_random_port(service).await;

    let res1 = crate::utils::get(&format!("http://{host}:{port}/home")).await;
    crate::utils::assert_content_type(&res1, "text/html");

    let res2 = crate::utils::get(&format!("http://{host}:{port}/home")).await;
    crate::utils::assert_content_type(&res2, "text/html");

    assert_eq!(values.lock().unwrap().clone(), vec![1, 2, 3, 1, 2, 3]);
    shutdown.send(()).unwrap();
}
