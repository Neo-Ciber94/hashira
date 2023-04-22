use wasm_bindgen::prelude::wasm_bindgen;
use js_sys::Iterator;

#[wasm_bindgen]
extern "C" {
    pub type Headers;

    #[wasm_bindgen(method)]
    pub fn entries(this: &Headers) -> Iterator;
    
    #[wasm_bindgen(method)]
    pub fn keys(this: &Headers) -> Iterator;

    #[wasm_bindgen(method)]
    pub fn values(this: &Headers) -> Iterator;
}