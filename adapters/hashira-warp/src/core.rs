use futures::stream::TryStreamExt;
use hashira::{
    app::AppService,
    types::TryBoxStream,
    web::{Body, Bytes, BytesMut, Payload, RemoteAddr, Request, Response},
};
use std::{convert::Infallible, fmt::Debug, net::SocketAddr};
use warp::{path::FullPath, reject::Reject, Buf, Filter, Stream};

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

fn with_service(
    service: AppService,
) -> impl Filter<Extract = (AppService,), Error = Infallible> + Clone {
    warp::any().map(move || service.clone())
}

// Main handler
fn hashira_filter(
    app_service: AppService,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    async fn handler(
        stream: impl Stream<Item = Result<impl Buf, warp::Error>> + Send + Sync + 'static,
        path: FullPath,
        headers: warp::hyper::HeaderMap,
        method: warp::http::Method,
        remote_addr: Option<SocketAddr>,
        service: AppService,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        {
            fn buf_to_stream(
                stream: impl Stream<Item = Result<impl Buf, warp::Error>> + Send + Sync + 'static,
            ) -> TryBoxStream<Bytes> {
                let ret = stream
                    .map_ok(|mut buf| {
                        let mut bytes = BytesMut::new();
                        buf.copy_to_slice(&mut bytes);
                        bytes.freeze()
                    })
                    .map_err(Into::into);

                Box::pin(ret)
            }

            let stream = buf_to_stream(stream);

            // Construct the incoming request
            let mut warp_req = try_reject!(Request::builder()
                .uri(path.as_str())
                .method(method)
                .body(Body::from(stream)));

            // Set the headers
            *warp_req.headers_mut() = headers;

            // Add remote address to the request
            if let Some(addr) = remote_addr {
                warp_req.extensions_mut().insert(RemoteAddr::from(addr));
            }

            // Send the request to hashira, and get the warp response
            let warp_res = try_reject!(handle_request(warp_req, service.clone()).await);
            Ok(warp_res)
        }
    }

    warp::any()
        .and(warp::body::stream())
        .and(warp::path::full())
        .and(warp::filters::header::headers_cloned())
        .and(warp::method())
        .and(warp::filters::addr::remote())
        .and(with_service(app_service))
        .and_then(handler)
}

/// Handles a `warp` request and returns a `warp` response.
pub async fn handle_request(
    warp_req: Request<Body>,
    app_service: AppService,
) -> Result<warp::hyper::Response<warp::hyper::Body>, hashira::error::BoxError> {
    let req = map_request(warp_req);
    let res = app_service.handle(req).await;
    let warp_res = map_response(res);
    Ok(warp_res)
}

#[inline(always)]
fn map_request(req: Request<Body>) -> Request {
    // Because the types are teh same we don't need to change anything,
    // the remote address extension is passed
    req
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
