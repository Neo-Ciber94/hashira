use super::{
    archive::ExtractOptions,
    global_cache::{FindVersion, GlobalCacheError},
    utils::cache_dir,
    Tool, Version,
};
use crate::tools::global_cache::GlobalCache;
use std::{path::PathBuf, str::FromStr};

pub struct Sass(PathBuf);

#[async_trait::async_trait]
impl Tool for Sass {
    fn name() -> &'static str {
        "sass"
    }

    fn binary_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "sass.bat"
        } else {
            "sass"
        }
    }

    fn default_version() -> super::Version {
        Version::new(1, 62, Some(0))
    }

    fn include() -> &'static [&'static str] {
        if cfg!(target_os = "windows") {
            &["sass.bat", "src/dart.exe", "src/sass.snapshot"]
        } else {
            &["sass", "src/dart", "src/sass.snapshot"]
        }
    }

    fn test_version_args() -> &'static [&'static str] {
        &["--version"]
    }

    fn parse_version(s: &str) -> anyhow::Result<super::Version> {
        // Version is on the format 0.0.0,
        let version_str = &s.trim();
        Version::from_str(version_str)
    }

    fn binary_path(&self) -> &std::path::Path {
        &self.0
    }

    async fn load_with_options(opts: super::LoadOptions<'_>) -> anyhow::Result<Self> {
        let version = opts.version.unwrap_or(Self::default_version()).to_string();
        let extract_opts = ExtractOptions {
            skip_base: true,
            preserve_dir: true,
        };

        match opts.install_dir {
            Some(dir) => {
                anyhow::ensure!(dir.is_dir(), "`{}` is not a directory", dir.display());
                let url = get_download_url(&version)?;
                let bin_path = GlobalCache::download::<Self>(&url, dir, extract_opts).await?;
                Ok(Self(bin_path))
            }
            None => {
                match GlobalCache::find_any::<Self>(FindVersion::Any).await {
                    Ok(bin_path) => Ok(Self(bin_path)),
                    Err(GlobalCacheError::NotFound(_)) => {
                        // Download and install
                        let url = get_download_url(&version)?;
                        let cache_path = cache_dir()?;
                        let bin_path =
                            GlobalCache::download::<Self>(&url, &cache_path, extract_opts).await?;
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
        ("windows", "x86_64") => format!("https://github.com/sass/dart-sass/releases/download/{version}/dart-sass-{version}-windows-x64.zip"),
        ("macos" | "linux", "x86_64") => format!("https://github.com/sass/dart-sass/releases/download/{version}/dart-sass-{version}-{target_os}-x64.tar.gz"),
        ("macos" | "linux", "aarch64") => format!("https://github.com/sass/dart-sass/releases/download/{version}/dart-sass-{version}-{target_os}-arm64.tar.gz"),
        _ => anyhow::bail!("Unable to download Sass for {target_os} {target_arch}")
      })
}
#[cfg(test)]
mod tests {
    use crate::tools::{sass::Sass, LoadOptions, Tool, ToolExt};

    #[tokio::test]
    async fn test_sass_download_and_version() {
        let temp_dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let download_path = temp_dir.into_path(); //temp_dir.path().to_path_buf();
        tokio::fs::create_dir_all(&download_path).await.unwrap();

        let wasm_bingen = Sass::load_with_options(LoadOptions {
            install_dir: Some(&download_path),
            version: Some(Sass::default_version()),
        })
        .await
        .unwrap();

        let version = wasm_bingen.test_version().unwrap();
        let default_version = Sass::default_version();
        assert_eq!(version, default_version)
    }
}
