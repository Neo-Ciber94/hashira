/// Name of the environment variable with the host where the app is running.
pub(crate) const HASHIRA_HOST: &str = "HASHIRA_HOST";

/// Name of the environment variable with the port where the app is running.
pub(crate) const HASHIRA_PORT: &str = "HASHIRA_PORT";

/// Name of the environment variable with the path where the static files are being served.
pub(crate) const HASHIRA_STATIC_DIR: &str = "HASHIRA_STATIC_DIR";

/// Name of the environment variable to check if live reload is enabled
pub(crate) const HASHIRA_LIVE_RELOAD: &str = "HASHIRA_LIVE_RELOAD";

/// Name of the environment variable with the host of the live reload.
pub(crate) const HASHIRA_LIVE_RELOAD_HOST: &str = "HASHIRA_LIVE_RELOAD_HOST";

/// Name of the environment variable with the port of the live reload.
pub(crate) const HASHIRA_LIVE_RELOAD_PORT: &str = "HASHIRA_LIVE_RELOAD_PORT";

/// Name of the environment variable with the name of the wasm library.
#[cfg_attr(target_arch="wasm32", allow(dead_code))]
pub(crate) const HASHIRA_WASM_LIB: &str = "HASHIRA_WASM_LIB";

/// Returns the name of the wasm client library.
#[cfg_attr(target_arch="wasm32", allow(dead_code))]
pub(crate) fn get_wasm_name() -> Option<String> {
    if let Ok(name) = std::env::var(HASHIRA_WASM_LIB) {
        return Some(name);
    }

    log::warn!("Unable to find wasm client library name, `HASHIRA_WASM_LIB` environment variable was not set");
    None
}

/// Returns the application host.
pub fn get_host() -> Option<String> {
    std::env::var(HASHIRA_HOST).ok()
}

/// Returns the application port.
pub fn get_port() -> Option<u16> {
    let port_str = std::env::var(HASHIRA_PORT).ok()?;
    match u16::from_str_radix(&port_str, 10) {
        Ok(port) => Some(port),
        Err(err) => {
            log::warn!("Failed to parse port number: {err}");
            None
        }
    }
}

/// Returns the application static dir.
pub fn get_static_dir() -> String {
    std::env::var(HASHIRA_STATIC_DIR).unwrap_or_else(|_| "/static".into())
}

/// Returns `true` if the application has live reload.
pub fn is_live_reload() -> bool {
    if let Ok(env) = std::env::var(HASHIRA_LIVE_RELOAD) {
        env == "1"
    } else {
        false
    }
}
