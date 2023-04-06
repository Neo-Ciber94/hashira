/// Name of the environment variable with the host where the app is running.
pub const HASHIRA_HOST: &str = "HASHIRA_HOST";

/// Name of the environment variable with the port where the app is running.
pub const HASHIRA_PORT: &str = "HASHIRA_PORT";

/// Name of the environment variable with the path where the static files are being served.
pub const HASHIRA_STATIC_DIR: &str = "HASHIRA_STATIC_DIR";

/// Returns the name of the crate running.
pub fn get_crate_name() -> Option<String> {
    fn crate_name() -> Option<String> {
        // By default we take the crate name from cargo
        if let Ok(name) = std::env::var("CARGO_PKG_NAME") {
            return Some(name);
        }

        if let Ok(exe_dir) = std::env::current_exe() {
            let file_name = exe_dir.file_stem().unwrap().to_str().unwrap().to_owned();
            return Some(file_name);
        }

        None
    }

    // We cleanup the name in case is in the form: `crate-name`
    match crate_name().map(|n| n.replace("-", "_")) {
        Some(name) => Some(name),
        None => {
            log::warn!("Unable to find crate name");
            None
        }
    }
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