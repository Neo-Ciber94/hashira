use hashira::adapter::Adapter;
use hashira_rocket::HashiraRocket;
use rocket::{fs::NamedFile, get, routes, Build, Rocket};
use {{crate_name}}::hashira;

pub async fn start_server() -> Result<(), hashira::error::Error> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app = hashira();
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
