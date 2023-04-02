use crate::components::{HelloPage, HelloPageProps, HomePage};
use ::hashira::server::{App as HashiraApp, AppService, Metadata};
use actix_files::{Files, NamedFile};
use actix_web::{get, web, App, HttpRequest, HttpServer, Responder};
use hashira::web::{Body, Response};
use yew::{html::ChildrenProps, BaseComponent};

pub async fn start_server<C>() -> std::io::Result<()>
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    let current_dir = get_current_dir();
    let host = "127.0.0.1";
    let port = 5000;
    let path = {
        let mut temp = current_dir.clone();
        temp.push("public");
        temp
    };

    println!("⚡ Server started at: http://{host}:{port}");
    println!("⚡ Serving static files from: {}", path.display());

    // Initialize hashira
    hashira::init();

    // Create and run the server
    HttpServer::new(move || {
        App::new()
            .service(favicon)
            .service(Files::new("static/", &path))
            .app_data(hashira::<C>())
            .service(
                web::resource("/{params:.*}").route(web::get().to(|req: HttpRequest| async {
                    hashira_actix_web::handle_request::<C>(req).await
                })),
            )
    })
    .bind((host, port))?
    .run()
    .await
}

#[get("/favicon.ico")]
async fn favicon() -> actix_web::Result<impl Responder> {
    let favicon = NamedFile::open_async("./public/favicon.ico").await?;
    Ok(favicon)
}

fn get_current_dir() -> std::path::PathBuf {
    let mut current_dir = std::env::current_exe().expect("failed to get current directory");
    current_dir.pop();
    current_dir
}

// Setup all the components
pub fn hashira<C>() -> AppService<C>
where
    C: BaseComponent<Properties = ChildrenProps>,
{
    HashiraApp::<C>::new()
        //.app_data(...)
        .page("/", |mut ctx| async {
            ctx.add_metadata(
                Metadata::new()
                    .viewport("width=device-width, initial-scale=1.0")
                    .title("Hashira Sample App | Counter")
                    .description("A counter made with hashira actix-web"),
            );

            let html = ctx.render::<HomePage>().await;
            Response::builder()
                .header(
                    hashira::web::header::CONTENT_TYPE,
                    "text/html; charset=utf-8",
                )
                .body(Body::from(html))
                .unwrap()
        })
        .page("/hello/:name", |mut ctx| async {
            let name = ctx.params().find("name").unwrap().to_owned();
            ctx.add_metadata(
                Metadata::new()
                    .viewport("width=device-width, initial-scale=1.0")
                    .title("Hashira Sample App | Hello")
                    .description("A hashira greeter"),
            );

            let html = ctx
                .render_with_props::<HelloPage>(HelloPageProps { name })
                .await;

            Response::builder()
                .header(
                    hashira::web::header::CONTENT_TYPE,
                    "text/html; charset=utf-8",
                )
                .body(Body::from(html))
                .unwrap()
        })
        .build()
}
