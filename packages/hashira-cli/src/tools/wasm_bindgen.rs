use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use super::{
    archive::ExtractBehavior,
    global_cache::{FindVersion, GlobalCache, GlobalCacheError},
    utils::cache_dir_path,
    Tool, Version,
};

#[derive(Clone)]
pub struct WasmBindgen(PathBuf);

#[async_trait::async_trait]
impl Tool for WasmBindgen {
    fn name() -> &'static str {
        "wasm-bindgen"
    }

    fn binary_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "wasm-bindgen.exe"
        } else {
            "wasm-bindgen"
        }
    }

    fn default_version() -> Version {
        Version::new(0, 2, Some(84))
    }

    fn test_version_args() -> &'static [&'static str] {
        &["--version"]
    }

    fn parse_version(s: &str) -> anyhow::Result<Version> {
        // Parses the version from the returned string,
        // is in the format: `wasm-bindgen 0.0.0`
        let Some(text) = s.trim().split(' ').nth(1) else {
            anyhow::bail!("unable to parse version string: `{s}`")
        };

        Version::from_str(text)
    }

    async fn load(dir: Option<&Path>) -> anyhow::Result<Self> {
        let version = Self::default_version().to_string();

        match dir {
            // Install in the given directory
            Some(dir) => {
                anyhow::ensure!(dir.is_dir(), "`{}` is not a directory", dir.display());
                let url = get_download_url(&version)?;
                let bin_path =
                    GlobalCache::install::<Self>(&url, dir, ExtractBehavior::SkipBasePath).await?;
                Ok(Self(bin_path))
            }

            // Install in the given directory
            None => {
                match GlobalCache::find_any::<Self>(FindVersion::Any).await {
                    Ok(bin_path) => Ok(Self(bin_path)),
                    Err(GlobalCacheError::NotFound(_)) => {
                        // Install
                        let url = get_download_url(&version)?;
                        let cache_path = cache_dir_path()?;
                        let bin_path = GlobalCache::install::<Self>(
                            &url,
                            &cache_path,
                            ExtractBehavior::SkipBasePath,
                        )
                        .await?;
                        Ok(Self(bin_path))
                    }
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }
        }
    }

    fn binary_path(&self) -> &Path {
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

#[cfg(test)]
mod tests {
    use crate::tools::{wasm_bindgen::WasmBindgen, Tool, ToolExt};

    #[tokio::test]
    async fn test_download_and_version() {
        let temp_dir = tempfile::tempdir().unwrap();
        let download_path = temp_dir.path().to_path_buf();
        tokio::fs::create_dir_all(&download_path).await.unwrap();

        let wasm_bingen = WasmBindgen::load(Some(&download_path)).await.unwrap();

        let version = wasm_bingen.test_version().unwrap();
        let default_version = WasmBindgen::default_version();
        assert_eq!(version, default_version)
    }

    #[tokio::test]
    async fn test_download_and_version_2() {
        let wasm_bingen = WasmBindgen::load(None).await.unwrap();

        let version = wasm_bingen.test_version().unwrap();
        let default_version = WasmBindgen::default_version();
        assert_eq!(version, default_version)
    }
}
