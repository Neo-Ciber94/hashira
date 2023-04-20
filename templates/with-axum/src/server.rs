use with_axum_client::hashira;
use yew::{html::ChildrenProps, BaseComponent};

pub async fn start_server<C>() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let host = hashira::env::get_host().unwrap_or_else(|| String::from("127.0.0.1"));
    let port = hashira::env::get_port().unwrap_or(5000);
    println!("⚡ Server started at: http://{host}:{port}");

    let app = axum::Router::new();
    let addr = format!("{host}:{port}").as_str().parse().unwrap();

    // Create and run the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

// #[actix_web::get("/favicon.ico")]
// async fn favicon() -> actix_web::Result<impl Responder> {
//     let favicon = NamedFile::open_async("./public/favicon.ico").await?;
//     Ok(favicon)
// }
