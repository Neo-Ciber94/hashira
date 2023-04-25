use super::{
    global_cache::{FindVersion, GlobalCacheError},
    utils::cache_dir_path,
    Tool, Version,
};
use crate::tools::{archive::ExtractBehavior, global_cache::GlobalCache};
use anyhow::Context;
use std::{path::PathBuf, str::FromStr};

pub struct TailwindCss(PathBuf);

#[async_trait::async_trait]
impl Tool for TailwindCss {
    fn binary_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "tailwindcss.exe"
        } else {
            "tailwindcss"
        }
    }

    fn default_version() -> super::Version {
        Version::new(3, 3, Some(1))
    }

    fn test_version_args() -> &'static [&'static str] {
        &["--help"]
    }

    fn parse_version(s: &str) -> anyhow::Result<super::Version> {
        // Is in the format: tailwindcss v0.0.0
        let line = s
            .lines()
            .map(|s| s.trim())
            .filter(|s| s.trim().len() > 0)
            .next()
            .with_context(|| format!("failed to parse tailwindcss version: {s:?}"))?;

        let text = line
            .split(' ')
            .nth(1)
            .with_context(|| format!("failed to parse tailwindcss version: {s:?}"))?;

        let version_text = text
            .strip_prefix('v')
            .context("failed to strip tailwindcss version `v`")?
            .trim();

        Version::from_str(version_text)
            .with_context(|| format!("failed to parse tailwindcss version: {s:?}"))
    }

    fn binary_path(&self) -> &std::path::Path {
        &self.0
    }

    async fn load(install_dir: Option<&std::path::Path>) -> anyhow::Result<Self> {
        let version = Self::default_version().to_string();

        match install_dir {
            Some(dir) => {
                anyhow::ensure!(dir.is_dir(), "`{}` is not a directory", dir.display());
                let url = get_download_url(&version)?;
                let bin_path =
                    GlobalCache::install::<Self>(&url, dir, ExtractBehavior::None).await?;
                Ok(Self(bin_path))
            }
            None => {
                match GlobalCache::find_any::<Self>(FindVersion::Any).await {
                    Ok(bin_path) => Ok(Self(bin_path)),
                    Err(GlobalCacheError::NotFound(_)) => {
                        // Download and install
                        let url = get_download_url(&version)?;
                        let cache_path = cache_dir_path()?;
                        let bin_path =
                            GlobalCache::install::<Self>(&url, &cache_path, ExtractBehavior::None)
                                .await?;
                        Ok(Self(bin_path))
                    }
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }
        }
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

    let target_arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        anyhow::bail!("unsupported target architecture")
    };

    Ok(match (target_os, target_arch) {
        ("windows", "x86_64") => format!("https://github.com/tailwindlabs/tailwindcss/releases/download/v{version}/tailwindcss-windows-x64.exe"),
        ("macos" | "linux", "x86_64") => format!("https://github.com/tailwindlabs/tailwindcss/releases/download/v{version}/tailwindcss-{target_os}-x64"),
        ("macos" | "linux", "aarch64") =>format!("https://github.com/tailwindlabs/tailwindcss/releases/download/v{version}/tailwindcss-{target_os}-arm64"),
        _ => anyhow::bail!("Unable to download tailwindcss for {target_os} {target_arch}")
    })
}

#[cfg(test)]
mod tests {
    use crate::tools::{tailwindcss::TailwindCss, Tool, ToolExt};

    #[tokio::test]
    async fn test_download_and_version() {
        let temp_dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let download_path = temp_dir.path().to_path_buf();
        tokio::fs::create_dir_all(&download_path).await.unwrap();

        let wasm_bingen = TailwindCss::load(Some(&download_path)).await.unwrap();

        let version = wasm_bingen.test_version().unwrap();
        let default_version = TailwindCss::default_version();
        assert_eq!(version, default_version)
    }
}
