#![cfg(not(feature = "client"))]

use hashira::app::AppService;
use once_cell::sync::Lazy;
use std::sync::Once;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

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
#[allow(unused_variables)]
pub fn set_envs(envs: js_sys::Object) {
    
    static ONCE: Once = Once::new();
    
    log::debug!("Setting envs: {:#?}", envs);

    //#[cfg(target_arch = "wasm32-unknown-unknown")]
    {
        use std::collections::HashMap;
        use wasm_bindgen::UnwrapThrowExt;

        ONCE.call_once(|| {
            let map =
                serde_wasm_bindgen::from_value::<HashMap<String, String>>(envs.into())
                    .expect_throw("failed to convert env to map");

            hashira::env::set_wasm_envs(map);
        });
    }
}
