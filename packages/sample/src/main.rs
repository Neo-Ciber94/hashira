use ::hashira::server::{App as HashiraApp, Metadata};
use actix_web::{get, web::Path, App, FromRequest, HttpRequest, HttpResponse, HttpServer};
use yew::Properties;

#[yew::function_component]
fn HelloPage() -> yew::Html {
    yew::html! {
        <h1>{"Hello World!"}</h1>
    }
}

#[derive(PartialEq, Properties, Clone)]
struct HelloNameProps {
    name: String,
}

#[yew::function_component]
fn HelloNamePage(props: &HelloNameProps) -> yew::Html {
    yew::html! {
        <h1>{format!("Hello {}!", props.name)}</h1>
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        let app = HashiraApp::<HttpRequest, HttpResponse>::new()
            .page("/", |mut ctx| async {
                ctx.add_metadata(
                    Metadata::new()
                        .viewport("width=device-width, initial-scale=1.0")
                        .title("Cool app")
                        .description("This is a really cool app"),
                );

                let html = ctx.render::<HelloPage>().await;
                HttpResponse::Ok().body(html)
            })
            .page("/hello/:name", |mut ctx| async {
                let request = ctx.request();
                let name = Path::<String>::extract(request).await.unwrap().into_inner();
                ctx.add_metadata(
                    Metadata::new()
                        .viewport("width=device-width, initial-scale=1.0")
                        .title("Cool app")
                        .description("This is a really cool app"),
                );

                let html = ctx
                    .render_with_props::<HelloNamePage>(HelloNameProps { name })
                    .await;
                HttpResponse::Ok().body(html)
            });

        let service = app.build();

        App::new().app_data(service).service(hashira)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/{params:.*}")]
async fn hashira(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    hashira_actix_web::handle_request(req).await
}
