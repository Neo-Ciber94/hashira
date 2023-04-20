use actix_files::{Files, NamedFile};
use actix_web::{
    cookie::Cookie, http::header, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use counter_dark_theme_web::hashira;
use yew::{html::ChildrenProps, BaseComponent};

pub async fn start_server<C>() -> std::io::Result<()>
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let host = hashira::env::get_host().unwrap_or_else(|| String::from("127.0.0.1"));
    let port = hashira::env::get_port().unwrap_or(5000);

    println!("⚡ Server started at: `http://{host}:{port}`");
    // Create and run the server
    HttpServer::new(move || {
        App::new()
            .service(favicon)
            .service(change_theme)
            .configure(hashira_actix_web::router(hashira::<C>()))
    })
    .bind((host, port))?
    .run()
    .await
}

#[actix_web::get("/api/change_theme")]
async fn change_theme(req: HttpRequest) -> impl Responder {
    let is_dark = req
        .cookie("dark")
        .map(|c| c.value() == "true")
        .unwrap_or_default();

    let location = req
        .headers()
        .get(header::REFERER)
        .cloned()
        .unwrap_or_else(|| header::HeaderValue::from_static("/"));

    let mut res = HttpResponse::TemporaryRedirect();
    let mut cookie = Cookie::new("dark", "true");
    cookie.set_path("/");

    if is_dark {
        cookie.make_removal();
    } else {
        cookie.make_permanent();
    }

    res.insert_header((header::LOCATION, location))
        .cookie(cookie)
        .finish()
}

#[actix_web::get("/favicon.ico")]
async fn favicon() -> actix_web::Result<impl Responder> {
    let favicon = NamedFile::open_async("./public/favicon.ico").await?;
    Ok(favicon)
}
