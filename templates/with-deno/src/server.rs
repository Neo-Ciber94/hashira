#![cfg(not(feature="client"))]

use hashira::app::AppService;
use once_cell::sync::Lazy;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use std::sync::Once;

use crate::App;

static HASHIRA: Lazy<AppService> = Lazy::new(|| crate::hashira::<App>());
static ONCE: Once = Once::new();

#[wasm_bindgen]
pub async fn handler(web_req: web_sys::Request) -> Result<web_sys::Response, JsValue> {
    ONCE.call_once(|| {
        wasm_logger::init(wasm_logger::Config::default());
        console_error_panic_hook::set_once();
    });    

    let service = HASHIRA.clone();
    let web_res = hashira_deno::core::handle_request(service, web_req).await;
    Ok(web_res)
}
