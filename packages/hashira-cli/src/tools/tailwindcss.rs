use super::{
    archive::ExtractOptions,
    global_cache::{install_tool, FindVersion, InstallToolOptions},
    LoadOptions, Tool, Version,
};
use anyhow::Context;
use std::{path::PathBuf, str::FromStr};

// Checkout: https://tailwindcss.com/

#[derive(Clone)]
pub struct TailwindCss(PathBuf);

#[async_trait::async_trait]
impl Tool for TailwindCss {
    fn name() -> &'static str {
        "tailwindcss"
    }

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
            .find(|s| !s.trim().is_empty())
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

    async fn load_with_options(opts: LoadOptions<'_>) -> anyhow::Result<Self> {
        let version = opts.version.unwrap_or(Self::default_version());
        let url = get_download_url(&version)?;
        let extract_opts = ExtractOptions::default();
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
        ("windows", "x86_64") => format!("https://github.com/tailwindlabs/tailwindcss/releases/download/v{version}/tailwindcss-windows-x64.exe"),
        ("macos" | "linux", "x86_64") => format!("https://github.com/tailwindlabs/tailwindcss/releases/download/v{version}/tailwindcss-{target_os}-x64"),
        ("macos" | "linux", "aarch64") =>format!("https://github.com/tailwindlabs/tailwindcss/releases/download/v{version}/tailwindcss-{target_os}-arm64"),
        _ => anyhow::bail!("Unable to download tailwindcss for {target_os} {target_arch}")
    })
}

#[cfg(test)]
mod tests {
    use crate::tools::{tailwindcss::TailwindCss, LoadOptions, Tool, ToolExt};

    #[tokio::test]
    async fn test_tailwind_download_and_version() {
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
