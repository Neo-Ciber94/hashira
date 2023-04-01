

pub fn hydrate() {
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("Hydrating app...");
    hashira::client::hydrate();
}
