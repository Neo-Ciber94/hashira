use actix_web::{web::Bytes, HttpRequest, HttpResponse, Resource};
use futures::TryStreamExt;
use hashira::{
    app::AppService,
    web::{Body, BodyInner, Request, Response},
};
use std::io::{Error, ErrorKind};

/// Returns a handler that matches all the requests.
pub fn router() -> Resource {
    actix_web::web::resource("/{params:.*}").to(|req: HttpRequest, body: Bytes| async {
        // We just forward the request and body to the handler
        handle_request(req, body).await
    })
}

/// Handle a request.
pub async fn handle_request(req: HttpRequest, body: Bytes) -> actix_web::Result<HttpResponse> {
    let path = req.path().to_string();
    let service = req
        .app_data::<AppService>()
        .cloned()
        .expect("Unable to find hashira `AppService`");

    let req = map_request(req, body).await?;
    let res = service.handle(req, &path).await;
    let actix_web_response = map_response(res);
    Ok(actix_web_response)
}

async fn map_request(src: HttpRequest, bytes: Bytes) -> actix_web::Result<Request> {
    let mut request = Request::builder()
        .uri(src.uri())
        .method(src.method())
        .version(src.version());

    let headers = request.headers_mut().unwrap();
    for (name, value) in src.headers() {
        headers.append(name, value.into());
    }

    let body = Body::from(bytes);
    request.body(body).map_err(actix_web::Error::from)
}

fn map_response(res: Response) -> HttpResponse {
    use actix_web::HttpResponseBuilder;

    let mut builder = HttpResponseBuilder::new(res.status());

    for (name, value) in res.headers() {
        builder.append_header((name, value));
    }

    match res.into_body().into_inner() {
        BodyInner::Bytes(bytes) => builder.body(bytes),
        BodyInner::Stream(stream) => {
            // We need to wrap the error on a sized type to be passed to `streaming`
            builder.streaming(stream.map_err(|err| Error::new(ErrorKind::Other, err)))
        }
    }
}
