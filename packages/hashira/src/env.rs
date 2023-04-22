use std::{collections::HashMap, sync::Mutex};

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
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(crate) const HASHIRA_WASM_LIB: &str = "HASHIRA_WASM_LIB";

/// Returns the name of the wasm client library.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(crate) fn get_wasm_name() -> Option<String> {
    return Some("with_deno".into());
    if let Some(name) = get_env(HASHIRA_WASM_LIB) {
        return Some(name);
    }

    log::warn!("Unable to find wasm client library name, `HASHIRA_WASM_LIB` environment variable was not set");
    None
}

/// Returns the application host.
pub fn get_host() -> Option<String> {
    get_env(HASHIRA_HOST)
}

/// Returns the application port.
pub fn get_port() -> Option<u16> {
    let port_str = get_env(HASHIRA_PORT)?;
    match port_str.parse::<u16>() {
        Ok(port) => Some(port),
        Err(err) => {
            log::warn!("Failed to parse port number: {err}");
            None
        }
    }
}

/// Returns the application static dir.
pub fn get_static_dir() -> String {
    get_env(HASHIRA_STATIC_DIR).unwrap_or_else(|| "/static".into())
}

/// Returns `true` if the application has live reload.
pub fn is_live_reload() -> bool {
    if let Some(env) = get_env(HASHIRA_LIVE_RELOAD) {
        env == "1"
    } else {
        false
    }
}

//#[cfg(target_arch="wasm32-unknown-unknown")]
static WASM_ENVS: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

/// Sets the environment variables of the wasm module.
//#[cfg(target_arch="wasm32-unknown-unknown")]
pub fn set_wasm_envs<I, K, V>(envs: I)
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    log::debug!("Setting wasm environment variables");

    let mut wasm_envs = WASM_ENVS.lock().unwrap();
    
    if wasm_envs.is_some() {
        panic!("wasm environment variables can only be set once");
    }

    let map = wasm_envs.get_or_insert_with(|| Default::default());

    for (key, value) in envs.into_iter() {
        map.insert(key.into(), value.into());
    }
}

fn get_env(name: impl AsRef<str>) -> Option<String> {
    #[cfg(not(target_arch = "wasm32-unknown-unknown"))]
    {
        std::env::var(name.as_ref()).ok()
    }

    #[cfg(target_arch = "wasm32-unknown-unknown")]
    {
        WASM_ENVS
            .get()
            .and_then(|envs| envs.get(name.as_ref()))
            .cloned()
    }
}
