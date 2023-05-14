use actix_files::Files;
use actix_web::{
    web::{self},
    HttpRequest, HttpResponse,
};
use futures::{StreamExt, TryStreamExt};
use hashira::{
    app::AppService,
    web::{Body, Payload, Request, Response},
};

/// Returns a function which adds a configuration to the actix web `App`
pub fn router(app_service: AppService) -> impl FnMut(&mut web::ServiceConfig) {
    move |cfg| {
        let serve_dir = get_current_dir().join("public");
        let static_dir = hashira::env::get_static_dir();

        cfg.app_data(app_service.clone())
            .service(Files::new(&static_dir, serve_dir))
            .default_service(web::to(
                |req: HttpRequest, body: actix_web::web::Payload| async {
                    // We just forward the request and body to the handler
                    handle_request(req, body).await
                },
            ));
    }
}

/// Returns a function which adds a configuration to the actix web `App` and handling the `hashira`
/// request at the given path.
pub fn router_with(path: &str, app_service: AppService) -> impl FnMut(&mut web::ServiceConfig) {
    let path = format!("{path}/{{params:.*}}");

    move |cfg| {
        let current_dir = get_current_dir().join("public");
        let static_dir = hashira::env::get_static_dir();

        cfg.app_data(app_service.clone())
            .service(Files::new(&static_dir, current_dir))
            .service(web::resource(&path).to(
                |req: HttpRequest, body: actix_web::web::Payload| async {
                    // We just forward the request and body to the handler
                    handle_request(req, body).await
                },
            ));
    }
}

/// Handle a request.
pub async fn handle_request(
    req: HttpRequest,
    body: actix_web::web::Payload,
) -> actix_web::Result<HttpResponse> {
    let service = req
        .app_data::<AppService>()
        .cloned()
        .expect("Unable to find hashira `AppService`");

    let req = map_request(req, body).await?;
    let res = service.handle(req).await;
    let actix_web_response = map_response(res);
    Ok(actix_web_response)
}

async fn map_request(
    actix_req: HttpRequest,
    mut payload: actix_web::web::Payload,
) -> actix_web::Result<Request> {
    let mut request = Request::builder()
        .uri(actix_req.uri())
        .method(actix_req.method())
        .version(actix_req.version());

    let headers = request.headers_mut().unwrap();
    for (name, value) in actix_req.headers() {
        headers.append(name, value.into());
    }

    // Sends the body as a stream
    let (sender, body) = Body::channel();

    actix_web::rt::spawn(async move {
        while let Some(chunk) = payload.next().await {
            let data = chunk.map_err(Into::into);
            if let Err(err) = sender.send(data) {
                log::error!("{:?}", err);
                break;
            }
        }
    });

    request.body(body).map_err(actix_web::Error::from)
}

fn map_response(res: Response) -> HttpResponse {
    use actix_web::HttpResponseBuilder;

    let mut builder = HttpResponseBuilder::new(res.status());

    for (name, value) in res.headers() {
        builder.append_header((name, value));
    }

    match res.into_body().into_inner() {
        Payload::Bytes(bytes) => builder.body(bytes),
        Payload::Stream(stream) => {
            // We need to wrap the error on a sized type to be passed to `streaming`
            builder.streaming(
                stream.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err)),
            )
        }
    }
}

fn get_current_dir() -> std::path::PathBuf {
    let mut current_dir = std::env::current_exe().expect("failed to get current directory");
    current_dir.pop();
    current_dir
}
