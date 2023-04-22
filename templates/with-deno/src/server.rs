use hashira::app::AppService;
use once_cell::sync::Lazy;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::App;

static HASHIRA: Lazy<AppService> = Lazy::new(|| crate::hashira::<App>());

#[wasm_bindgen]
pub async fn handler(web_req: web_sys::Request) -> Result<web_sys::Response, JsValue> {
    console_error_panic_hook::set_once();

    let service = HASHIRA.clone();
    let req = hashira_deno::core::map_request(web_req)
        .await
        .expect("failed to map request");
    let res = service.handle(req).await;
    let web_res = hashira_deno::core::map_response(res)
        .await
        .expect("failed to map response");

    Ok(web_res)
}
