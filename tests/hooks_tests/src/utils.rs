use hashira::app::{App as Hashira, AppService};
use std::net::SocketAddr;
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
    Hashira::<App>::new().page::<HelloWorldPage>()
}

#[hashira::page_component("/home")]
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

    // router with hashira
    let router = hashira_axum::core::router(service);

    // Signals for shutdown
    let (tx_shutdown, rx_shutdown) = tokio::sync::oneshot::channel::<()>();

    // Signals for server started
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

pub fn assert_content_type(res: &reqwest::Response, content_type: &str) {
    let header = res
        .headers()
        .get("content-type")
        .unwrap_or_else(|| panic!("content type not found: {:#?}", res.headers()));

    assert!(header.to_str().unwrap().starts_with(content_type));
}

// Just make a GET request
pub async fn get(url: &str) -> reqwest::Response {
    reqwest::get(url).await.expect("request failed")
}
