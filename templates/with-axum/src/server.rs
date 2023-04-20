use with_axum_client::hashira;
use hashira_axum::HashiraAxum;
use yew::{html::ChildrenProps, BaseComponent};

pub async fn start_server<C>() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app = hashira::<C>();
    HashiraAxum::new().serve(app).await
}

// #[actix_web::get("/favicon.ico")]
// async fn favicon() -> actix_web::Result<impl Responder> {
//     let favicon = NamedFile::open_async("./public/favicon.ico").await?;
//     Ok(favicon)
// }
