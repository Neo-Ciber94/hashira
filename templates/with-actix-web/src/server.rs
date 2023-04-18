use actix_files::NamedFile;
use actix_web::{App, HttpServer, Responder};
use with_actix_web_client::hashira;
use yew::{html::ChildrenProps, BaseComponent};

pub async fn start_server<C>() -> std::io::Result<()>
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let host = hashira::env::get_host().unwrap_or_else(|| String::from("127.0.0.1"));
    let port = hashira::env::get_port().unwrap_or(5000);
    log::info!("⚡ Server started at: http://{host}:{port}");

    // Create and run the server
    HttpServer::new(move || {
        App::new()
            .service(favicon)
            .configure(hashira_actix_web::router(hashira::<C>()))
    })
    .bind((host, port))?
    .run()
    .await
}

// Serves the favicon
#[actix_web::get("/favicon.ico")]
async fn favicon() -> actix_web::Result<impl Responder> {
    let favicon = NamedFile::open_async("./public/favicon.ico").await?;
    Ok(favicon)
}
