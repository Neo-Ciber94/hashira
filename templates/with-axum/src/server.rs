use axum::Router;
use hashira::adapter::Adapter;
use hashira_axum::HashiraAxum;
use tower_http::services::ServeDir;
use {{crate_name}}::hashira;
use yew::{html::ChildrenProps, BaseComponent};

pub async fn start_server<BASE>() -> Result<(), hashira::error::Error>
where
    BASE: BaseComponent<Properties = ChildrenProps>,
{
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app = hashira::<BASE>();
    HashiraAxum::from(axum()).serve(app).await
}

fn axum() -> Router {
    Router::new().route(
        "/favicon.ico",
        axum::routing::get_service(ServeDir::new("./public")),
    )
}
