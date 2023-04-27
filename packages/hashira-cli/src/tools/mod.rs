// Tools
pub mod node_js;
pub mod npm;
pub mod npx;
pub mod parcel;
pub mod sass;
pub mod tailwindcss;
pub mod wasm_bindgen;

//
use std::{
    ffi::OsStr,
    fmt::Display,
    path::Path,
    process::{Child, Command},
    str::FromStr,
};

use anyhow::Context;

pub(crate) mod archive;
pub(crate) mod global_cache;
pub(crate) mod utils;

#[derive(Default, Debug, Clone)]
pub struct LoadOptions<'a> {
    pub version: Option<Version>,
    pub install_dir: Option<&'a Path>,
}

/// An external tool.
#[async_trait::async_trait]
pub trait Tool: Sized {
    /// Name of this tool, mainly used for debugging.
    fn name() -> &'static str;

    /// Name of the executable binary of this tool.
    fn binary_name() -> &'static str;

    /// Returns the default version of this tool.
    fn default_version() -> Version;

    /// Returns the arguments used to execute the test command of this tool.
    fn test_version_args() -> &'static [&'static str];

    /// Parses the version of this tool from the given string.
    fn parse_version(s: &str) -> anyhow::Result<Version>;

    /// All the files to include from the downloaded file when installing this tool.
    /// If empty, will try to include the value specified by the `binary_name`,
    /// otherwise all the files to include including the actual executable must be declared here.
    fn include() -> &'static [&'static str] {
        &[]
    }

    /// Returns the path to the executable.
    fn binary_path(&self) -> &Path;

    /// Loads the tool from cache or install it.
    async fn load() -> anyhow::Result<Self> {
        Self::load_with_options(Default::default()).await
    }

    /// Loads the tool from cache or install it using the given options.
    async fn load_with_options(opts: LoadOptions<'_>) -> anyhow::Result<Self>;

    /// Returns the actual path of the executable to include.
    fn binary_include_path() -> &'static str {
        let include = Self::include();
        if include.is_empty() {
            Self::binary_name()
        } else {
            let bin_name = Self::binary_name();

            for file in include {
                let name = Path::new(file).components().last().unwrap();

                if let Some(name) = name.as_os_str().to_str() {
                    if name == bin_name {
                        return file;
                    }
                }
            }

            panic!("`{bin_name}` was not found within the included files")
        }
    }

    // The binary name should exists if we declare the include files
    #[doc(hidden)]
    fn assert_include_files() -> anyhow::Result<()> {
        let include = Self::include();
        if include.is_empty() {
            return Ok(());
        }

        let bin_name = Self::binary_name();
        let mut binary_included = false;

        for file in include {
            let name = Path::new(file)
                .components()
                .last()
                .with_context(|| format!("failed to read include file `{file}`"))?;

            if let Some(name) = name.as_os_str().to_str() {
                if name == bin_name {
                    binary_included = true;
                    break;
                }
            }
        }

        anyhow::ensure!(
            binary_included,
            "binary `{}` is not declared within the included files",
            bin_name
        );

        Ok(())
    }
}

pub trait ToolExt: Tool {
    /// Test the version of this tool.
    fn test_version(&self) -> anyhow::Result<Version> {
        let args = Self::test_version_args();
        let output = self
            .cmd()
            .args(args)
            .output()
            .with_context(|| format!("failed to run: {}", self.binary_path().display()))?;

        let result = String::from_utf8_lossy(&output.stdout);
        Self::parse_version(&result)
    }

    /// Spawn a command with the given args and returns the child process.
    fn spawn<I, S>(&self, args: I) -> anyhow::Result<Child>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let child = self
            .cmd()
            .args(args)
            .spawn()
            .with_context(|| format!("failed to run: {}", self.binary_path().display()))?;
        Ok(child)
    }

    /// Returns a command to execute this tool
    fn cmd(&self) -> Command {
        let bin_path = self.binary_path();
        Command::new(bin_path)
    }

    /// Returns a asynchronous command to execute this tool
    fn async_cmd(&self) -> tokio::process::Command {
        let bin_path = self.binary_path();
        tokio::process::Command::new(bin_path)
    }
}

/// Test the version of the given binary without any checks if the path actually matches this binary name
pub(crate) fn unchecked_test_version<T: Tool>(
    binary_path: impl AsRef<Path>,
) -> anyhow::Result<Version> {
    let binary_path = binary_path.as_ref();

    anyhow::ensure!(
        binary_path.exists(),
        "binary could not be found: {}",
        binary_path.display()
    );

    let version_args = T::test_version_args();
    let output = Command::new(binary_path)
        .args(version_args)
        .output()
        .with_context(|| format!("failed to run: {}", binary_path.display()))?;

    let version_text = String::from_utf8_lossy(&output.stdout);
    let version = T::parse_version(&version_text)?;
    Ok(version)
}

impl<T> ToolExt for T where T: Tool {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    mayor: u32,
    minor: u32,
    patch: Option<u32>,
}

impl Version {
    pub fn new(mayor: u32, minor: u32, patch: Option<u32>) -> Self {
        Version {
            mayor,
            minor,
            patch,
        }
    }

    // FIXME: getters?
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.patch {
            Some(patch) => write!(
                f,
                "{mayor}.{minor}.{patch}",
                mayor = self.mayor,
                minor = self.minor
            ),
            None => write!(f, "{mayor}.{minor}", mayor = self.mayor, minor = self.minor),
        }
    }
}

impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('.').collect::<Vec<_>>();
        anyhow::ensure!(
            parts.len() >= 2 && parts.len() <= 3,
            "invalid string, expected at least 3 digits, but was `{s:?}`"
        );

        let mayor = parts[0]
            .parse()
            .with_context(|| format!("invalid `mayor` in version string: {s:?}"))?;
        let minor = parts[1]
            .parse()
            .with_context(|| format!("invalid `minor` in version string: {s:?}"))?;
        let patch = if parts.len() == 3 {
            let patch = parts[2]
                .parse()
                .with_context(|| format!("invalid `patch` in version string: {s:?}"))?;
            Some(patch)
        } else {
            None
        };

        Ok(Version::new(mayor, minor, patch))
    }
}
