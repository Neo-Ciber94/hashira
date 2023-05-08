use hashira::adapter::Adapter;
use hashira_warp::HashiraWarp;
use warp::Filter;
use with_warp::hashira;

pub async fn start_server() -> Result<(), hashira::error::Error> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app = hashira();
    HashiraWarp::from(warp()).serve(app).await
}

fn warp() -> impl warp::Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone  {
    warp::path("favicon.ico")
        .and(warp::fs::file("./public/favicon.ico"))
}
