use actix_files::{Files, NamedFile};
use actix_web::{App, HttpServer, Responder, middleware};
use actix_web_sample_web::hashira;
use yew::{html::ChildrenProps, BaseComponent};

pub async fn start_server<C>() -> std::io::Result<()>
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let current_dir = get_current_dir().join("public");
    let host = hashira::env::get_host().unwrap_or_else(|| String::from("127.0.0.1"));
    let port = hashira::env::get_port().unwrap_or(5000);
    let static_dir = hashira::env::get_static_dir();

    println!("⚡ Server started at: http://{host}:{port}");
    println!(
        "⚡ Serving static files from: `{}` to `{static_dir}`",
        current_dir.display()
    );

    // Create and run the server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::NormalizePath::trim())
            .service(favicon)
            .service(Files::new(&static_dir, &current_dir))
            .app_data(hashira::<C>())
            .service(hashira_actix_web::router())
    })
    .bind((host, port))?
    .run()
    .await
}

#[actix_web::get("/favicon.ico")]
async fn favicon() -> actix_web::Result<impl Responder> {
    let favicon = NamedFile::open_async("./public/favicon.ico").await?;
    Ok(favicon)
}

fn get_current_dir() -> std::path::PathBuf {
    let mut current_dir = std::env::current_exe().expect("failed to get current directory");
    current_dir.pop();
    current_dir
}
