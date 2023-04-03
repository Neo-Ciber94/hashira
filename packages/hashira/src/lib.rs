pub mod app;
pub mod client;
pub mod components;
pub mod error;
pub mod server;
pub mod web;

/// Initialize the hashira framework.
#[cfg(not(target_arch = "wasm32"))]
pub fn init() {
    std::env::set_var(consts::HASHIRA_INIT, "1");
}

/// Returns `true` if the framework is initialized.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn is_initialized() -> bool {
    if let Ok(value) = std::env::var(crate::consts::HASHIRA_INIT) {
        return value == "1";
    }

    return false;
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod consts {
    pub const HASHIRA_INIT: &str = "HASHIRA_INIT";
}
