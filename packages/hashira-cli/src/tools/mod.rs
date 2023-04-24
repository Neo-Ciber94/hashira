use std::path::{Path, PathBuf};

pub(crate) mod decompress;
pub(crate) mod global_cache;
pub(crate) mod utils;

// Tools
mod wasm_bindgen;

/// Defines how to install an external tool
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum Installation {
    /// Only install if not is already in cache
    #[default]
    IfRequired,

    /// Install this tool in the given path,
    /// useful for testing
    #[allow(dead_code)]
    Target(PathBuf),
}

#[derive(Default, Debug, Clone)]
pub struct InstallOptions {
    pub installation: Installation,
}

/// An external tool.
#[async_trait::async_trait]
pub trait Tool: Sized {
    /// Returns the name of this tool.
    fn name() -> &'static str;

    /// Name of the executable binary of this tool.
    fn bin_name() -> &'static str;

    /// Returns the version of this tool.
    fn version() -> &'static str;

    /// Returns the path to the executable.
    fn bin(&self) -> &Path;

    /// Executes this tool and get the version.
    async fn test_version(&self) -> anyhow::Result<String>;

    /// Install this tool if is not already installed and return the path to the executable.
    async fn get(opts: InstallOptions) -> anyhow::Result<Self>;
}
