use axum::{response::IntoResponse, routing::get_service, Extension, Router};
use hashira::{
    app::AppService,
    web::{Body, BodyInner, Request, Response},
};
use hyper::{body::to_bytes, StatusCode};
use tower_http::services::ServeDir;

// Returns a router for a `Axum` application.
pub fn router(app_service: AppService) -> Router {
    let static_dir = hashira::env::get_static_dir();
    let serve_dir = get_current_dir().join("public");

    Router::new()
        .nest_service(&static_dir, get_service(ServeDir::new(serve_dir)))
        .fallback(handle_request)
        .layer(Extension(app_service))
}

/// Handle a request.
pub async fn handle_request(
    Extension(service): Extension<AppService>,
    axum_request: Request<axum::body::Body>,
) -> impl IntoResponse {
    match map_request(axum_request).await {
        Ok(req) => {
            let res = service.handle(req).await;
            map_response(res)
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

async fn map_request(req: Request<axum::body::Body>) -> Result<Request, axum::Error> {
    let (parts, body) = req.into_parts();
    let body = Body::from(to_bytes(body).await.map_err(axum::Error::new)?);
    Ok(Request::from_parts(parts, body))
}

fn map_response(res: Response) -> axum::response::Response {
    let (parts, body) = res.into_parts();
    let body = match body.into_inner() {
        BodyInner::Bytes(bytes) => axum::body::Body::from(bytes),
        BodyInner::Stream(stream) => axum::body::Body::wrap_stream(stream),
    };

    axum::response::Response::from_parts(parts, axum::body::boxed(body))
}

fn get_current_dir() -> std::path::PathBuf {
    let mut current_dir = std::env::current_exe().expect("failed to get current directory");
    current_dir.pop();
    current_dir
}
