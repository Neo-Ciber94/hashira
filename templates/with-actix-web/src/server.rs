use actix_files::NamedFile;
use actix_web::{web::ServiceConfig, Responder};
use hashira::adapter::Adapter;
use hashira_actix_web::HashiraActixWeb;
use yew::{html::ChildrenProps, BaseComponent};
use {{crate_name}}::hashira;

pub async fn start_server<BASE>() -> Result<(), hashira::error::Error>
where
    BASE: BaseComponent<Properties = ChildrenProps>,
{
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let app = hashira::<BASE>();
    HashiraActixWeb::from(actix_web).serve(app).await
}

fn actix_web(cfg: &mut ServiceConfig) {
    cfg.service(favicon);
}

// Serves the favicon
#[actix_web::get("/favicon.ico")]
async fn favicon() -> actix_web::Result<impl Responder> {
    let favicon = NamedFile::open_async("./public/favicon.ico").await?;
    Ok(favicon)
}
