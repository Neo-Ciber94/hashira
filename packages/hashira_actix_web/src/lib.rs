use actix_web::{HttpRequest, HttpResponse};
use hashira::server::AppService;

pub async fn handle_request<C>(req: HttpRequest) -> actix_web::Result<HttpResponse>
where
    C: 'static,
{
    let path = req.path().to_string();
    let service = req
        .app_data::<AppService<HttpRequest, HttpResponse, C>>()
        .cloned()
        .expect("Unable to find hashira `AppService`");

    let res = service.handle(req, &path).await;
    Ok(res)
}
