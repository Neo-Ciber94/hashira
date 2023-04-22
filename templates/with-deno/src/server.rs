#![cfg(not(feature = "client"))]

use hashira::app::AppService;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Once};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue, UnwrapThrowExt};

use crate::App;

static HASHIRA: Lazy<AppService> = Lazy::new(|| crate::hashira::<App>());
static ONCE: Once = Once::new();

/// Handle the given request and returns a response.
#[wasm_bindgen]
pub async fn handler(web_req: web_sys::Request) -> Result<web_sys::Response, JsValue> {
    ONCE.call_once(|| {
        wasm_logger::init(wasm_logger::Config::default());
        console_error_panic_hook::set_once();
    });

    let service = HASHIRA.clone();
    let web_res = hashira_wasm::core::handle_request(service, web_req).await;
    Ok(web_res)
}

/// Initializes the environment variables of the application.
#[wasm_bindgen]
pub fn set_envs(envs: JsValue) {
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        let map = serde_wasm_bindgen::from_value::<HashMap<String, String>>(envs)
            .expect_throw("failed to convert env to map");

        for (key, value) in map {
            std::env::set_var(key, value);
        }
    });
}
