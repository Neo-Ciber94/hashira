use crate::components::{HelloPage, HelloPageProps, HomePage};
use ::hashira::server::{App as HashiraApp, AppService, Metadata};
use actix_files::{Files, NamedFile};
use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer};

pub async fn start_server() -> std::io::Result<()> {
    let host = "127.0.0.1";
    let port = 5000;
    let path = {
        let mut temp = std::env::current_exe().expect("failed to get current directory");
        temp.pop();
        temp.push("public");
        temp
    };

    println!("⚡ Server started at: http://{host}:{port}");
    println!("⚡ Serving static files from: {}", path.display());

    // Initialize hashira
    hashira::init();

    // Create and run the server
    HttpServer::new(move || {
        let favicon = NamedFile::open("./public/favicon.ico").expect("unable to find favicon");

        App::new()
            .service(favicon)
            .service(Files::new("static/", &path))
            .app_data(hashira())
            .service(hashira_router)
    })
    .bind((host, port))?
    .run()
    .await
}

// Actix web adapter
#[get("/{params:.*}")]
async fn hashira_router(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    hashira_actix_web::handle_request(req).await
}

// Setup all the components
pub fn hashira() -> AppService<HttpRequest, HttpResponse> {
    HashiraApp::<HttpRequest, HttpResponse>::new()
        .page("/", |mut ctx| async {
            ctx.add_metadata(
                Metadata::new()
                    .viewport("width=device-width, initial-scale=1.0")
                    .title("Hashira Sample App | Counter")
                    .description("A counter made with hashira actix-web"),
            );

            let html = ctx.render::<HomePage>().await;
            HttpResponse::Ok().body(html)
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
            HttpResponse::Ok().body(html)
        })
        .build()
}
