use hashira::adapter::Adapter;
use hashira_rocket::HashiraRocket;
use multipart_upload::hashira;
use rocket::{fs::NamedFile, get, routes, Build, Rocket};

pub async fn start_server() -> Result<(), hashira::error::Error> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app = hashira();
    HashiraRocket::from(rocket()).serve(app).await?;
    Ok(())
}

fn rocket() -> Rocket<Build> {
    let uploads = std::env::current_exe().unwrap().parent().unwrap().join("uploads");
    Rocket::build()
        .mount("/", routes![favicon])
        .mount("/uploads", rocket::fs::FileServer::from(uploads))
}

#[get("/favicon.ico")]
pub async fn favicon() -> Option<NamedFile> {
    NamedFile::open("./public/favicon.ico").await.ok()
}

// use axum::Router;
// use hashira::adapter::Adapter;
// use hashira_axum::HashiraAxum;

// use multipart_upload::hashira;
// use tower_http::services::ServeDir;

// pub async fn start_server() -> Result<(), hashira::error::Error> {
//     env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

//     let app = hashira();
//     HashiraAxum::from(axum()).serve(app).await
// }

// fn axum() -> Router {
//     let uploads = std::env::current_exe()
//         .unwrap()
//         .parent()
//         .unwrap()
//         .join("uploads");

//     Router::new()
//         .nest_service(
//             "/uploads",
//             axum::routing::get_service(ServeDir::new(uploads)),
//         )
//         .route(
//             "/favicon.ico",
//             axum::routing::get_service(ServeDir::new("./public")),
//         )
// }
