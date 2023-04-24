use std::path::{Path, PathBuf};

use super::{
    global_cache::{GlobalCache, GlobalCacheError},
    InstallOptions, Tool,
};

#[derive(Clone)]
pub struct WasmBindgen(PathBuf);

#[async_trait::async_trait]
impl Tool for WasmBindgen {
    fn name() -> &'static str {
        "wasm-bingen"
    }

    fn bin_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "wasm-bingen.exe"
        } else {
            "wasm-bingen"
        }
    }

    fn version() -> &'static str {
        "0.2.84"
    }

    async fn test_version(&self) -> anyhow::Result<String> {
        let bin_path = GlobalCache::find(Self::bin_name()).await?;

        // Parses the version from the returned string,
        // is in the format: `wasm-bingen 0.0.00`
        let version_text = super::utils::exec_and_get_output(bin_path, ["--version"])?;
        let Some(version) = version_text.strip_prefix("wasm-bingen ") else {
            anyhow::bail!("unable to parse version string")
        };

        Ok(version.to_owned())
    }

    async fn get(opts: InstallOptions) -> anyhow::Result<Self> {
        let version = Self::version();
        let bin_name = Self::bin_name();

        match opts.installation {
            // Get from cache or install
            super::Installation::IfRequired => {
                let expected_version = format!("{} {}", Self::name(), version);
                let args = ["--version"];

                match GlobalCache::find_any(bin_name, args, &expected_version).await {
                    Ok(bin_path) => {
                        // Returns from cache
                        Ok(Self(bin_path))
                    }
                    Err(GlobalCacheError::NotFound(_)) => {
                        // Install
                        let url = get_download_url(version)?;
                        let bin_path = GlobalCache::install(bin_name, &url, None).await?;
                        Ok(Self(bin_path))
                    }
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }

            // Install in the given directory
            super::Installation::Target(dir) => {
                anyhow::ensure!(dir.is_dir(), "`{}` is not a directory", dir.display());
                let url = get_download_url(version)?;
                let bin_path = GlobalCache::install(bin_name, &url, Some(dir)).await?;
                Ok(Self(bin_path))
            }
        }
    }

    fn bin(&self) -> &Path {
        self.0.as_path()
    }
}

fn get_download_url(version: &str) -> anyhow::Result<String> {
    let target_os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        anyhow::bail!("unsupported OS")
    };

    let os = match target_os {
        "windows" => "pc-windows-msvc",
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-musl",
        _ => unreachable!(),
    };

    Ok(format!("https://github.com/rustwasm/wasm-bindgen/releases/download/{version}/wasm-bindgen-{version}-x86_64-{os}.tar.gz"))
}
