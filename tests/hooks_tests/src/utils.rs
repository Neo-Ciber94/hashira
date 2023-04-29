use std::net::SocketAddr;
use hashira::app::{App as Hashira, AppService};
use yew::html::ChildrenProps;

#[yew::function_component]
pub fn App(props: &ChildrenProps) -> yew::Html {
    yew::html! {
       <>
        {for props.children.iter()}
       </>
    }
}

pub fn create_app() -> Hashira<App> {
    Hashira::<App>::new().page("/home", |ctx| async {
        let res = ctx.render::<HelloWorldPage, _>().await;
        Ok(res)
    })
}

#[hashira::page_component]
fn HelloWorldPage() -> yew::Html {
    yew::html! {
        "Hello World"
    }
}

pub struct ServerHandle {
    pub shutdown: tokio::sync::oneshot::Sender<()>,
    pub host: String,
    pub port: u16,
}

pub async fn start_server_in_random_port(service: AppService) -> ServerHandle {
    let host = "127.0.0.1";
    let port = portpicker::pick_unused_port().unwrap();
    let addr: SocketAddr = format!("{host}:{port}").as_str().parse().unwrap();

    let router = hashira_axum::core::router(service);

    let (tx_shutdown, rx_shutdown) = tokio::sync::oneshot::channel::<()>();
    let (tx_server_start, rx_server_start) = tokio::sync::oneshot::channel::<()>();
    let server = axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .with_graceful_shutdown(async move {
            rx_shutdown.await.ok();
        });

    tokio::spawn(async move {
        tx_server_start.send(()).unwrap();
        server.await.expect("failed to start server");
    });

    rx_server_start.await.ok(); // Wait for the server to start

    ServerHandle {
        shutdown: tx_shutdown,
        host: host.to_owned(),
        port,
    }
}
