use crate::components::{HomePage, HelloPage};

pub fn hydrate() {
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("Hydrating app...");
    hashira::client::hydrate::<HomePage>();
    hashira::client::hydrate::<HelloPage>();
}
