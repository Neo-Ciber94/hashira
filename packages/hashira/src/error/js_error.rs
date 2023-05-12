use thiserror::Error;
use wasm_bindgen::{JsValue, JsCast};

/// A javascript error.
#[derive(Debug, Error)]
#[error("{0}")]
pub struct JsError(String);

impl JsError {
    pub fn new(err: JsValue) -> Self {
        use std::fmt::Write;

        if let Some(s) = err.as_string() {
            return JsError(s);
        }

        match err.dyn_into::<js_sys::Error>() {
            Ok(err) => JsError(err.to_string().into()),
            Err(err) => {
                #[cfg(target_arch = "wasm32")]
                {
                    match serde_wasm_bindgen::from_value::<String>(err.clone()) {
                        Ok(str) => JsError(str),
                        Err(_) => {
                            let mut buf = String::new();
                            write!(buf, "{err:?}").expect("failed to format javascript error");
                            JsError(buf)
                        }
                    }
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    let mut buf = String::new();
                    write!(buf, "{err:?}").expect("failed to format javascript error");
                    JsError(buf)
                }
            }
        }
    }
}
