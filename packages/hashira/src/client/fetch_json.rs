#[cfg(target_arch = "wasm32")]
pub async fn fetch_json<S: serde::de::DeserializeOwned>(
    request: web_sys::Request,
) -> Result<S, crate::error::Error> {
    use crate::error::JsError;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(JsError::new)?;

    let resp: web_sys::Response = resp_value.dyn_into().unwrap();

    if !resp.ok() {
        return Err(get_response_error(resp).await);
    }

    // Convert this other `Promise` into a rust `Future`.
    let json = resp.json().map_err(JsError::new)?;
    let json = JsFuture::from(json).await.map_err(JsError::new)?;

    match serde_wasm_bindgen::from_value(json) {
        Ok(x) => Ok(x),
        Err(err) => {
            let s = err.to_string();
            Err(s.into())
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn get_response_error(resp: web_sys::Response) -> crate::error::Error {
    use wasm_bindgen_futures::JsFuture;
    debug_assert!(!resp.ok());

    let content_type = resp
        .headers()
        .get("content-type")
        .ok()
        .flatten()
        .unwrap_or_default();

    let error_message = match content_type.as_str() {
        "application/json" => {
            let json = resp.json().unwrap();
            let json = JsFuture::from(json).await.unwrap();
            serde_wasm_bindgen::from_value::<String>(json).unwrap()
        }
        _ => {
            let text = resp.text().unwrap();
            let text = JsFuture::from(text).await.unwrap();
            text.as_string().unwrap()
        }
    };

    error_message.into()
}
