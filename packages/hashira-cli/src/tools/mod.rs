use std::path::PathBuf;

pub(crate) mod decompress;
pub(crate) mod utils;

// Tools
mod wasm_bingen;

/// Defines how to install an external tool
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum Installation {
    /// Only install if not is already in cache
    #[default]
    IfRequired,

    /// Install even is already installed
    Force,

    /// Not install and get from cache
    NoInstall,

    /// Install this tool in the given path,
    /// useful for testing
    ToPath(PathBuf),
}

#[derive(Default, Debug, Clone)]
pub struct InstallOptions {
    pub installation: Installation,
}

/// An external tool.
#[async_trait::async_trait]
pub trait Tool: Sized {
    /// Returns the name of this tool.
    fn name(&self) -> &'static str;

    /// Returns the version of this tool.
    fn version(&self) -> &'static str;

    /// Executes this tool and get the version.
    async fn test_version(&self) -> anyhow::Result<String>;

    /// Install this tool if is not already installed and return the path to the executable.
    async fn get(opts: InstallOptions) -> anyhow::Result<Self>;

    /// Execute this tool with the given arguments.
    async fn exec(&self, args: Vec<String>) -> anyhow::Result<()>;
}
