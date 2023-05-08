use std::{path::PathBuf, str::FromStr};

use super::{
    archive::ExtractOptions,
    global_cache::{install_tool, FindVersion, InstallToolOptions},
    Tool, Version,
};

pub struct CargoGenerate(PathBuf);

#[async_trait::async_trait]
impl Tool for CargoGenerate {
    fn name() -> &'static str {
        "cargo generate"
    }

    fn binary_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "cargo-generate.exe"
        } else {
            "cargo-generate"
        }
    }

    fn default_version() -> super::Version {
        Version::new(0, 18, Some(2))
    }

    fn test_version_args() -> &'static [&'static str] {
        &["--version"]
    }

    fn parse_version(s: &str) -> anyhow::Result<super::Version> {
        // version is on the formats: cargo generate-generate x.xx.x, cargo generate x.xx.x
        let s: &str = s.trim();

        if let Some(text) = s.strip_prefix("cargo generate") {
            return Version::from_str(text.trim());
        }

        if let Some(text) = s.strip_prefix("cargo generate-generate") {
            return Version::from_str(text.trim());
        }

        anyhow::bail!("failed to parse version of: {s}")
    }

    fn binary_path(&self) -> &std::path::Path {
        &self.0
    }

    async fn load_with_options(opts: super::LoadOptions<'_>) -> anyhow::Result<Self> {
        let extract_opts = ExtractOptions::default();
        let url = get_url(Self::default_version())?;
        let opts = InstallToolOptions {
            dest: opts.install_dir,
            extract_opts,
            find_version: FindVersion::Any,
            min_version: Some(Self::default_version()),
            url: url.as_str(),
        };

        let bin_path = install_tool::<Self>(opts).await?;
        Ok(Self(bin_path))
    }
}

fn get_url(version: Version) -> anyhow::Result<String> {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        anyhow::bail!("unsupported os");
    };

    let target_arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        anyhow::bail!("unsupported target architecture")
    };

    Ok(match (os, target_arch) {
        ("windows", "x86_64") => format!("https://github.com/cargo-generate/cargo-generate/releases/download/v{version}/cargo-generate-v{version}-{target_arch}-pc-windows-msvc.tar.gz"),
        ("macos", "x86_64") | ("macos", "aarch64")=> format!("https://github.com/cargo-generate/cargo-generate/releases/download/v{version}/cargo-generate-v{version}-{target_arch}-apple-darwin.tar.gz"),
        ("linux", "x86_64") | ("linux", "aarch64")=> format!("https://github.com/cargo-generate/cargo-generate/releases/download/v{version}/cargo-generate-v{version}-{target_arch}-unknown-linux-gnu.tar.gz"),
        _ => anyhow::bail!("unsupported target architecture {os} {target_arch}"),
    })
}

#[cfg(test)]
mod tests {
    use crate::tools::{tailwindcss::TailwindCss, LoadOptions, Tool, ToolExt};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cargo_generate_download_and_version() {
        let temp_dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let download_path = temp_dir.path().to_path_buf();
        tokio::fs::create_dir_all(&download_path).await.unwrap();

        let wasm_bingen = TailwindCss::load_with_options(LoadOptions {
            install_dir: Some(temp_dir.path()),
            ..Default::default()
        })
        .await
        .unwrap();

        let version = wasm_bingen.test_version().unwrap();
        let default_version = TailwindCss::default_version();
        assert_eq!(version, default_version)
    }
}
