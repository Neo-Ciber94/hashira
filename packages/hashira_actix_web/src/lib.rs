use actix_web::{HttpRequest, HttpResponse};
use hashira::server::AppService;

pub async fn handle_request(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let path = req.path();
    let service = req
        .app_data::<AppService<HttpRequest, HttpResponse>>()
        .cloned()
        .expect("Unable to find hashira `AppService`");
    
    let page = service.router().recognize(&path).expect("not found"); // TODO: Return 404
    let ctx = service.create_context(req);
    let res = page.handler().call(ctx).await;
    Ok(res)
}
