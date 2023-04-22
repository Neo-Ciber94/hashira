use js_sys::Iterator;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Headers)]
    pub type Headers;

    #[wasm_bindgen(method, js_class = "Headers", js_name = entries)]
    pub fn entries(this: &Headers) -> Iterator;
}