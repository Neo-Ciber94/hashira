pub mod app;
pub mod client;
pub mod components;
pub mod error;
pub mod server;
pub mod web;

//
pub(crate) mod consts;
pub use server::env::Environment;

/// Initialize the hashira framework.
#[cfg(not(target_arch = "wasm32"))]
pub fn init() {
    // TODO: Set the variable checking other condition
    std::env::set_var(consts::HASHIRA_ENV, Environment::Development.to_string());
}

/// Returns `true` if the framework is initialized.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn is_initialized() -> bool {
    use std::str::FromStr;

    if let Ok(s) = std::env::var(crate::consts::HASHIRA_ENV) {
        match Environment::from_str(&s) {
            Ok(env) => {
                return matches!(env, Environment::Development | Environment::Production);
            }
            _ => {}
        }
    }

    return false;
}
