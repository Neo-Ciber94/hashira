use hashira::adapter::Adapter;
use hashira_tide::HashiraTide;
use {{crate_name}}::hashira;

pub async fn start_server() -> Result<(), hashira::error::BoxError> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app = hashira();
    HashiraTide::from(tide()).serve(app).await
}

fn tide() -> tide::Server<()>  {
    let mut server = tide::new();
    server.at("/favicon.ico").serve_file("./public/favicon.ico").expect("failed to serve favicon.ico");
    server
}
