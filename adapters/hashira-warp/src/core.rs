use hashira::{
    app::AppService,
    web::{Body, Payload, Bytes, Request, Response},
};
use std::fmt::Debug;
use warp::{path::FullPath, reject::Reject, Filter};

struct HashiraRejection(Box<dyn std::error::Error + Send + Sync>);
impl Debug for HashiraRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl Reject for HashiraRejection {}

// Helper for rejections
macro_rules! try_reject {
    ($input:expr) => {{
        match $input {
            Ok(x) => x,
            Err(err) => return Err(warp::reject::custom(HashiraRejection(err.into()))),
        }
    }};
}

// Returns base filter for a `Warp` application.
pub fn router(
    app_service: AppService,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let static_dir = hashira::env::get_static_dir();
    let serve_dir = get_current_dir().join("public");

    // Serves the static files or send the request to hashira
    warp::path(static_dir.trim_start_matches("/").to_owned())
        .and(warp::fs::dir(serve_dir))
        .or(hashira_filter(app_service))
}

// Main handler
fn hashira_filter(
    app_service: AppService,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::any()
        .and(warp::body::bytes())
        .and(warp::path::full())
        .and(warp::filters::header::headers_cloned())
        .and(warp::method())
        .and_then(
            move |bytes: Bytes,
                  path: FullPath,
                  headers: warp::hyper::HeaderMap,
                  method: warp::http::Method| {
                let service = app_service.clone();
                async move {
                    // Construct the incoming request
                    let mut warp_req = try_reject!(Request::builder()
                        .uri(path.as_str())
                        .method(method)
                        .body(bytes));

                    // Set the headers
                    *warp_req.headers_mut() = headers;

                    // Send the request to hashira, and get the warp response
                    let warp_res = try_reject!(handle_request(warp_req, service.clone()).await);
                    Ok(warp_res)
                }
            },
        )
}

/// Handles a `warp` request and returns a `warp` response.
pub async fn handle_request(
    warp_req: warp::hyper::Request<Bytes>,
    app_service: AppService,
) -> Result<warp::hyper::Response<warp::hyper::Body>, hashira::error::BoxError> {
    let req = map_request(warp_req).await?;
    let res = app_service.handle(req).await;
    let warp_res = map_response(res);
    Ok(warp_res)
}

async fn map_request(req: Request<Bytes>) -> Result<Request, hashira::error::BoxError> {
    let (parts, bytes) = req.into_parts();
    let body = Body::from(bytes);
    Ok(Request::from_parts(parts, body))
}

fn map_response(res: Response) -> warp::hyper::Response<warp::hyper::Body> {
    let (parts, body) = res.into_parts();
    let body = match body.into_inner() {
        Payload::Bytes(bytes) => warp::hyper::Body::from(bytes),
        Payload::Stream(stream) => warp::hyper::Body::wrap_stream(stream),
    };

    warp::hyper::Response::from_parts(parts, body)
}

fn get_current_dir() -> std::path::PathBuf {
    let mut current_dir = std::env::current_exe().expect("failed to get current directory");
    current_dir.pop();
    current_dir
}
