use hashira::adapter::Adapter;
use hashira_rocket::HashiraRocket;
use multipart_upload::hashira;
use rocket::{
    data::{Limits, ToByteUnit},
    fs::NamedFile,
    get, routes, Build, Config, Rocket,
};

pub async fn start_server() -> Result<(), hashira::error::BoxError> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app = hashira();
    HashiraRocket::from(rocket()).serve(app).await?;
    Ok(())
}

fn rocket() -> Rocket<Build> {
    let uploads = {
        cfg_if::cfg_if! {
            if #[cfg(not(feature = "client"))] {
               multipart_upload::uploads_dir()
            } else {
                unreachable!()
            }
        }
    };

    // Update the limits for the files
    let limits = Limits::new()
        .limit("bytes", 3.megabytes());

    let config = Config {
        limits,
        ..Default::default()
    };

    Rocket::build()
        .configure(config)
        .mount("/", routes![favicon])
        .mount("/uploads", rocket::fs::FileServer::from(uploads))
}

#[get("/favicon.ico")]
pub async fn favicon() -> Option<NamedFile> {
    NamedFile::open("./public/favicon.ico").await.ok()
}