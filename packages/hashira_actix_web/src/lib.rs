use actix_web::{HttpRequest, HttpResponse};
use hashira::server::AppService;

pub async fn handle_request(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let path = req.path().to_string();
    let service = req
        .app_data::<AppService<HttpRequest, HttpResponse>>()
        .cloned()
        .expect("Unable to find hashira `AppService`");

    let res = service.handle(req, &path).await;
    Ok(res)
}
