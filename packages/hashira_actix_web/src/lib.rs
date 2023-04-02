use actix_web::{web::Bytes, FromRequest, HttpRequest, HttpResponse};
use hashira::{
    server::AppService,
    web::{Body, Request, Response},
};

pub async fn handle_request<C>(req: HttpRequest) -> actix_web::Result<HttpResponse>
where
    C: 'static,
{
    let path = req.path().to_string();
    let service = req
        .app_data::<AppService<C>>()
        .cloned()
        .expect("Unable to find hashira `AppService`");

    let req = map_request(req).await?;
    let res = service.handle(req, &path).await;
    let actix_web_response = map_response(res);
    Ok(actix_web_response)
}

async fn map_request(src: HttpRequest) -> actix_web::Result<Request> {
    let bytes = Bytes::extract(&src).await?;
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
