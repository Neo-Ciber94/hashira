use futures::StreamExt;
use hashira::{
    app::AppService,
    web::{
        header::{HeaderName, HeaderValue},
        method::Method,
        uri::Uri,
        Body, Bytes, Request, Response,
    },
};
use std::{collections::HashMap, str::FromStr};
use wasm_bindgen::{JsCast, JsError, JsValue};
use web_sys::ResponseInit;

/// Handle a request.
#[allow(clippy::let_and_return)]
pub async fn handle_request(
    service: AppService,
    web_req: web_sys::Request,
) -> Result<web_sys::Response, JsError> {
    // Map the request to a `hashira`
    let req = crate::core::map_request(web_req).await?;

    // Get the `hashira` response
    let res = service.handle(req).await;

    // Map the response to `wasm`
    let web_res = crate::core::map_response(res).await?;

    // Return the response
    Ok(web_res)
}

async fn map_request(web_req: web_sys::Request) -> Result<Request, JsError> {
    let method = Method::from_str(&web_req.method()).expect("invalid method");
    let uri = Uri::from_str(&web_req.url()).expect("invalid uri");

    let mut builder = Request::builder().method(method).uri(uri);
    let req_headers = web_req
        .headers()
        .unchecked_into::<super::bindings::headers::Headers>();

    // Headers::entries()
    let header_entries = req_headers.entries();

    // SAFETY: headers is iterable
    let iterator = js_sys::try_iter(header_entries.as_ref())
        .map_err(map_js_error("failed to get headers iterator"))?;

    if let Some(iterator) = iterator {
        for array in iterator {
            let array: JsValue =
                array.map_err(map_js_error("failed to convert headers to array"))?;

            let array = array
                .dyn_into::<js_sys::Array>()
                .expect("failed to cast header to array");

            // SAFETY: Header array will always had 2 values
            let key_str = array.at(0).as_string().unwrap();
            let value_str = array.at(1).as_string().unwrap();

            let key = HeaderName::from_str(&key_str)?;
            let value = HeaderValue::from_str(&value_str)?;
            builder = builder.header(key, value);
        }
    }

    let bytes = match web_req.body() {
        Some(s) => {
            let readable = s.dyn_into().unwrap(); // SAFETY: Is already a stream
            let mut stream = wasm_streams::ReadableStream::from_raw(readable).into_stream();
            let (sender, body) = Body::channel();

            fn value_to_string(value: JsValue) -> hashira::Result<String> {
                value
                    .as_string()
                    .ok_or_else(|| format!("failed to convert chunk to string").into())
            }

            wasm_bindgen_futures::spawn_local(async move {
                while let Some(js) = stream.next().await {
                    let chunk = js
                        .map_err(js_to_error)
                        .and_then(value_to_string)
                        .map(Bytes::from);

                    if let Err(err) = sender.send(chunk) {
                        log::error!("{:?}", err);
                        break;
                    }
                }
            });

            body
        }
        None => Body::empty(),
    };

    let req = builder.body(bytes)?;
    Ok(req)
}

async fn map_response(res: Response) -> Result<web_sys::Response, JsError> {
    let (parts, body) = res.into_parts();
    let body = body
        .into_bytes()
        .await
        .map_err(|err| JsError::new(&err.to_string()))?;

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

    let headers = serde_wasm_bindgen::to_value(&map)?;
    init.headers(&headers);

    let res = web_sys::Response::new_with_opt_u8_array_and_init(Some(&mut bytes), &init)
        .map_err(map_js_error("failed to create response"))?;

    Ok(res)
}

fn map_js_error(details: impl Into<String>) -> impl FnOnce(JsValue) -> JsError {
    fn js_error(details: &str, js_error: JsValue) -> JsError {
        if js_error.is_string() {
            return JsError::new(&js_error.as_string().unwrap());
        }

        let error = serde_wasm_bindgen::from_value::<String>(js_error)
            .expect("failed to convert js to string");
        let message = format!("{details}, {error}");
        JsError::new(&message)
    }

    let details = details.into();
    move |err| {
        let s = details.as_str();
        js_error(s, err)
    }
}

fn js_to_error(js: JsValue) -> hashira::error::BoxError {
    use std::fmt::Write;

    if let Some(str) = js.as_string() {
        return str.into();
    }

    if let Some(err) = js.dyn_ref::<js_sys::Error>() {
        let msg = err.message();
        return msg.as_string().unwrap().into(); // SAFETY: The value is an string
    }

    let mut buf = String::new();
    write!(buf, "{js:?}").expect("failed to write error message");
    buf.into()
}
