use actix_web::{HttpRequest, HttpResponse};
use hashira::server::AppService;

pub async fn handle_request(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let path = req.path();
    let service = req
        .app_data::<AppService<HttpRequest, HttpResponse>>()
        .cloned()
        .expect("Unable to find hashira `AppService`");

    let mtch = service.router().recognize(&path).expect("not found"); // TODO: Return 404
    let params = mtch.params().clone();
    let ctx = service.create_context(req, params);
    let res = mtch.handler().call(ctx).await;
    Ok(res)
}
