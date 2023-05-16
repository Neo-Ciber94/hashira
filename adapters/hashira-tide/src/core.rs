use std::{net::SocketAddr, str::FromStr};

use futures::stream::TryStreamExt;

use hashira::{
    app::AppService,
    web::{Payload, RemoteAddr, Request, Response, ResponseExt},
};

// Returns a router for a `Tide` application.
pub fn router(app_service: AppService) -> tide::Server<()> {
    router_with_state(app_service, ())
}

// Returns a router with the given state for a `Tide` application.
pub fn router_with_state<S>(app_service: AppService, state: S) -> tide::Server<S>
where
    S: Clone + Send + Sync + 'static,
{
    let server = tide::with_state(state);
    with_router(server, app_service)
}

// Attach the hashira routes to an existing tide server.
#[doc(hidden)]
pub fn with_router<S>(mut server: tide::Server<S>, app_service: AppService) -> tide::Server<S>
where
    S: Clone + Send + Sync + 'static,
{
    let static_dir = hashira::env::get_static_dir();
    let serve_dir = get_current_dir().join("public");

    server
        .at(&static_dir)
        .serve_dir(serve_dir)
        .expect("failed to serve dir");

    // Tide do not allow to catch all routes
    // https://github.com/http-rs/tide/issues/295
    // So we catch the root, and any other path separately

    server.at("/").all({
        let service = app_service.clone();
        move |req: tide::Request<S>| {
            let service = service.clone();
            async move {
                let service = service.clone();
                let res = handle_request(req, service).await?;
                Ok(res)
            }
        }
    });

    server.at("*").all(move |req: tide::Request<S>| {
        let service = app_service.clone();
        async move {
            let service = service.clone();
            let res = handle_request(req, service).await?;
            Ok(res)
        }
    });

    server
}

/// Handle a request.
pub async fn handle_request<S>(
    tide_req: tide::Request<S>,
    app_service: AppService,
) -> Result<tide::Response, tide::Error> {
    let req = map_request(tide_req).await?;
    let res = app_service.handle(req).await;
    let tide_res = map_response(res)?;
    Ok(tide_res)
}

async fn map_request<S>(mut tide_req: tide::Request<S>) -> Result<Request, tide::Error> {
    let body = tide_req.take_body();
    let stream = hashira::internal::reader_stream::to_stream(body);
    let mut req = Request::builder().uri(tide_req.url().as_str());

    if let Some(v) = tide_req.version() {
        let version = match v {
            tide::http::Version::Http0_9 => hashira::web::version::Version::HTTP_09,
            tide::http::Version::Http1_0 => hashira::web::version::Version::HTTP_10,
            tide::http::Version::Http1_1 => hashira::web::version::Version::HTTP_11,
            tide::http::Version::Http2_0 => hashira::web::version::Version::HTTP_2,
            tide::http::Version::Http3_0 => hashira::web::version::Version::HTTP_3,
            s => panic!("unknown http version: {s}"),
        };

        req = req.version(version);
    }

    for name in tide_req.header_names() {
        if let Some(value) = tide_req.header(name) {
            let name = name.as_str();
            let value = value.as_str();
            req = req.header(name, value);
        }
    }

    // Add additional extensions
    let remote_addr = tide_req
        .remote()
        .and_then(|s| SocketAddr::from_str(s).ok())
        .map(RemoteAddr::from);

    if let Some(remote_addr) = remote_addr {
        req = req.extension(remote_addr);
    }

    let req = req.body(hashira::web::Body::from(stream))?;

    Ok(req)
}

fn map_response(res: Response) -> Result<tide::Response, tide::Error> {
    let cookies = res.cookies().clone()?;
    let (parts, body) = res.into_parts();
    let body = match body.into_inner() {
        Payload::Bytes(bytes) => tide::Body::from_bytes(bytes.to_vec()),
        Payload::Stream(stream) => {
            let s = stream.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
            let reader = s.into_async_read();

            // We set the length to zero to specify is chunked
            // https://docs.rs/tide/latest/tide/struct.Body.html#method.from_reader
            tide::Body::from_reader(reader, None)
        }
    };

    let mut tide_res = tide::Response::new(parts.status.as_u16());
    tide_res.set_body(body);

    // We insert each cookie to sync with tide,
    // because the cookies are not added to the header
    for c in cookies {
        let cookie = tide::http::Cookie::parse(c.to_string())?;
        tide_res.insert_cookie(cookie);
    }

    let mut last_header = None;
    for (key, value) in parts.headers {
        if let Some(key) = key {
            last_header = Some(key);
        }

        // SAFETY: The first header will always a value
        let name = last_header.as_ref().unwrap();
        tide_res.append_header(name.as_str(), value.to_str()?);
    }

    Ok(tide_res)
}

fn get_current_dir() -> std::path::PathBuf {
    let mut current_dir = std::env::current_exe().expect("failed to get current directory");
    current_dir.pop();
    current_dir
}
