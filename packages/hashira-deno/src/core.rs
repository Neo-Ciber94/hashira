use std::{collections::HashMap, str::FromStr};

use futures::StreamExt;
use hashira::{
    app::AppService,
    error::Error,
    web::{
        header::{HeaderName, HeaderValue},
        method::Method,
        uri::Uri,
        Body, Request, Response,
    },
};
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use web_sys::{console, ResponseInit};

// Returns a router for a `Deno` application.
// pub fn router(app_service: AppService) -> Box<dyn Fn(web_sys::Request) -> web_sys::Response> {
//     // Use Deno bindings
//     // let static_dir = hashira::env::get_static_dir();
//     // let serve_dir = get_current_dir().join("public");

//     Box::new(move |web_req| {

//     })
// }

/// Handle a request.
pub async fn handle_request(service: AppService, web_req: web_sys::Request) -> web_sys::Response {
    let req = crate::core::map_request(web_req)
        .await
        .expect("failed to map request");
    let res = service.handle(req).await;
    let web_res = crate::core::map_response(res)
        .await
        .expect("failed to map response");

    web_res
}

async fn map_request(web_req: web_sys::Request) -> Result<Request, hashira::error::Error> {
    let method = Method::from_str(&web_req.method()).expect("invalid method");
    let uri = Uri::from_str(&web_req.url()).expect("invalid uri");

    let mut builder = Request::builder().method(method).uri(uri);

    // SAFETY: headers is iterable
    let iterator = js_sys::try_iter(web_req.headers().as_ref()).unwrap();

    if let Some(iterator) = iterator {
        for array in iterator {
            let array = array
                .map_err(|js| js.as_string())
                .map_err(|js| {
                    js.unwrap_or_else(|| {
                        String::from("Something went wrong extracting the header values")
                    })
                })
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

            let array = array
                .dyn_into::<js_sys::Array>()
                .expect("failed to cast header to array");
            let key_str = array.at(0).as_string().unwrap();
            let value_str = array.at(1).as_string().unwrap();

            let key = HeaderName::from_str(&key_str)?;
            let value = HeaderValue::from_str(&value_str)?;
            builder = builder.header(key, value);
        }
    }

    let bytes = match web_req.body() {
        Some(s) => {
            let readable = s.dyn_into().unwrap();
            let mut stream = wasm_streams::ReadableStream::from_raw(readable).into_stream();

            let mut bytes = vec![];
            while let Some(js) = stream.next().await {
                let chunk = js.expect_throw("invalid chunk returned");
                let chunk_str = chunk
                    .as_string()
                    .expect("failed to convert chunk to string");

                console::log_1(&chunk);
                bytes.extend(chunk_str.as_bytes());
            }

            Body::from(bytes)
        }
        None => Body::empty(),
    };

    let req = builder.body(bytes)?;
    Ok(req)
}

async fn map_response(res: Response) -> Result<web_sys::Response, Error> {
    let (parts, body) = res.into_parts();
    let body = body.into_bytes().await.unwrap();
    let mut bytes = body.to_vec();

    let mut init = ResponseInit::new();
    init.status(parts.status.as_u16());

    let mut map = HashMap::new();
    for key in parts.headers.keys() {
        let name = key.to_string();
        let values = parts
            .headers
            .get_all(&name)
            .iter()
            .filter_map(|n| n.to_str().ok())
            .map(|s| s.to_owned())
            .collect::<Vec<_>>()
            .join(";");
        map.insert(name, values);
    }

    let headers = serde_wasm_bindgen::to_value(&map)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
    init.headers(&headers);

    let res =
        web_sys::Response::new_with_opt_u8_array(Some(&mut bytes)).map_err(js_value_to_error)?;

    Ok(res)
}

// fn get_current_dir() -> std::path::PathBuf {
//     let mut current_dir = std::env::current_exe().expect("failed to get current directory");
//     current_dir.pop();
//     current_dir
// }

fn js_value_to_error(js_value: JsValue) -> Error {
    let Some(str) = js_value.as_string() else {
        return std::io::Error::new(std::io::ErrorKind::Other, "wasm error").into();
    };

    std::io::Error::new(std::io::ErrorKind::Other, str).into()
}
