use actix_files::NamedFile;
use actix_web::{
    cookie::Cookie, http::header, web::ServiceConfig, HttpRequest, HttpResponse, Responder,
};
use counter_dark_theme_web::hashira;
use hashira::adapter::Adapter;
use hashira_actix_web::HashiraActixWeb;
use yew::{html::ChildrenProps, BaseComponent};

pub async fn start_server<BASE>() -> Result<(), hashira::error::Error>
where
    BASE: BaseComponent<Properties = ChildrenProps>,
{
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app = hashira::<BASE>();
    HashiraActixWeb::from(actix_web).serve(app).await
}

fn actix_web(cfg: &mut ServiceConfig) {
    cfg.service(favicon).service(change_theme);
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
