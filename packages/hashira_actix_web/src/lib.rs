use actix_web::{web::Bytes, HttpRequest, HttpResponse, Resource};
use hashira::{
    app::AppService,
    web::{Body, Request, Response},
};

/// Returns a handler that matches all the requests.
pub fn router() -> Resource {
    actix_web::web::resource("/{params:.*}")
        .to(|req: HttpRequest, bytes: Bytes| async { handle_request(req, bytes).await })
}

/// Handle a request.
pub async fn handle_request(req: HttpRequest, bytes: Bytes) -> actix_web::Result<HttpResponse> {
    let path = req.path().to_string();
    let service = req
        .app_data::<AppService>()
        .cloned()
        .expect("Unable to find hashira `AppService`");

    let req = map_request(req, bytes).await?;
    let res = service.handle(req, &path).await;
    let actix_web_response = map_response(res);
    Ok(actix_web_response)
}

async fn map_request(src: HttpRequest, bytes: Bytes) -> actix_web::Result<Request> {
    let body = Body::from(bytes);

    let mut request = Request::builder()
        .uri(src.uri())
        .method(src.method())
        .version(src.version());

    let headers = request.headers_mut().unwrap();
    for (name, value) in src.headers() {
        headers.append(name, value.into());
    }

    request.body(body).map_err(|e| actix_web::Error::from(e))
}

fn map_response(res: Response) -> HttpResponse {
    use actix_web::HttpResponseBuilder;

    let mut builder = HttpResponseBuilder::new(res.status());

    for (name, value) in res.headers() {
        builder.insert_header((name, value));
    }

    let body = res.into_body();
    let bytes = Bytes::from(body);
    builder.body(bytes)
}
