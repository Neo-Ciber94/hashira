#![cfg(not(feature = "client"))]

use hashira::app::AppService;
use once_cell::sync::Lazy;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

static HASHIRA: Lazy<AppService> = Lazy::new(crate::hashira);

// Entry point of the library
#[wasm_bindgen(start, skip_typescript)]
pub fn entry() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
}

/// Handle the given request and returns a response.
#[wasm_bindgen]
pub async fn handler(web_req: web_sys::Request) -> Result<web_sys::Response, JsValue> {
    let service = HASHIRA.clone();
    let web_res = hashira_wasm::core::handle_request(service, web_req).await?;
    Ok(web_res)
}

/// Initializes the environment variables of the application.
#[wasm_bindgen]
#[cfg(target_arch = "wasm32")]
pub fn set_envs(envs: js_sys::Object) {
    use std::collections::HashMap;
    use std::sync::Once;
    use wasm_bindgen::UnwrapThrowExt;

    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        let map = serde_wasm_bindgen::from_value::<HashMap<String, String>>(envs.into())
            .expect_throw("failed to convert env to map");

        log::debug!("Setting envs");
        hashira::env::wasm::set_envs(map);
    });
}
