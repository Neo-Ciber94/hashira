use std::{cell::RefCell, sync::Arc, pin::Pin};

use axum::{response::IntoResponse, Extension, Router};
use axum_macros::debug_handler;
use futures::{FutureExt, TryFutureExt, task::FutureObj};
use hashira::{
    app::AppService,
    web::{Body, Request, Response}, components::error::NotFoundPage,
};
use hyper::{body::to_bytes, StatusCode};

pub fn router(app_service: AppService) -> Router {
    Router::new()
        .layer(Extension(app_service))
        .fallback(handle_request)
}

/// Handle a request.
#[debug_handler]
pub async fn handle_request(axum_request: Request<axum::body::Body>) -> impl IntoResponse {
    let path = axum_request.uri().path().to_string();
    let service = axum_request
        .extensions()
        .get::<Extension<AppService>>()
        .cloned()
        .expect("Unable to find hashira `AppService`");

    match map_request(axum_request).await {
        Ok(req) => {
            let res = service.handle(req, &path).await;
            map_response(res)
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

async fn map_request(mut req: Request<axum::body::Body>) -> Result<Request, axum::Error> {
    let mut builder = Request::builder()
        .version(req.version())
        .method(req.method())
        .uri(req.uri());

    if let Some(headers) = builder.headers_mut() {
        *headers = std::mem::take(req.headers_mut());
    }

    if let Some(ext) = builder.extensions_mut() {
        *ext = std::mem::take(req.extensions_mut());
    }

    let axum_body = req.into_body();
    let bytes = to_bytes(axum_body).await.map_err(axum::Error::new)?;
    let ret = builder.body(Body::from(bytes)).map_err(axum::Error::new)?;
    Ok(ret)
}

fn map_response(mut res: Response) -> axum::response::Response {
    let mut builder = axum::response::Response::builder()
        .version(res.version())
        .status(res.status());

    if let Some(headers) = builder.headers_mut() {
        *headers = std::mem::take(res.headers_mut());
    }

    if let Some(ext) = builder.extensions_mut() {
        *ext = std::mem::take(res.extensions_mut());
    }

    let body = match res.into_body().into_inner() {
        hashira::web::BodyInner::Bytes(bytes) => axum::body::Body::from(bytes),
        hashira::web::BodyInner::Stream(stream) => axum::body::Body::wrap_stream(stream),
    };

    builder.body(axum::body::boxed(body)).unwrap()
}
