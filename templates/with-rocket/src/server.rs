use hashira::adapter::Adapter;
use hashira_rocket::HashiraRocket;
use rocket::{fs::NamedFile, get, routes, Build, Rocket};
use with_rocket_client::hashira;
use yew::{html::ChildrenProps, BaseComponent};

pub async fn start_server<BASE>() -> Result<(), hashira::error::Error>
where
    BASE: BaseComponent<Properties = ChildrenProps>,
{
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app = hashira::<BASE>();
    HashiraRocket::from(rocket()).serve(app).await?;
    Ok(())
}

fn rocket() -> Rocket<Build> {
    Rocket::build().mount("/", routes![favicon])
}

#[get("/favicon.ico")]
pub async fn favicon() -> Option<NamedFile> {
    NamedFile::open("./public/favicon.ico").await.ok()
}
