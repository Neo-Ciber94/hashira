use super::{
    archive::ExtractOptions,
    global_cache::{install_tool, FindVersion, InstallToolOptions},
    Tool, Version,
};
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
        let version = opts.version.unwrap_or(Self::default_version());
        let url = get_download_url(&version)?;
        let extract_opts = ExtractOptions {
            skip_base: true,
            preserve_dir: true,
        };

        let install_opts = InstallToolOptions {
            dest: opts.install_dir,
            extract_opts,
            find_version: FindVersion::Any,
            min_version: None,
            url: url.as_str(),
        };

        let bin_path = install_tool::<Self>(install_opts).await?;
        Ok(Self(bin_path))
    }
}

fn get_download_url(version: &Version) -> anyhow::Result<String> {
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

    #[tokio::test(flavor = "multi_thread")]
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
