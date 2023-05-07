use axum::Router;
use hashira::adapter::Adapter;
use hashira_axum::HashiraAxum;

use todo_app_sqlite::hashira;
use tower_http::services::ServeDir;

pub async fn start_server() -> Result<(), hashira::error::Error> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app = hashira().await?;
    HashiraAxum::from(axum()).serve(app).await
}

fn axum() -> Router {
    Router::new().route(
        "/favicon.ico",
        axum::routing::get_service(ServeDir::new("./public")),
    )
}
