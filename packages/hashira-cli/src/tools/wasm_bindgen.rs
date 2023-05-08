use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use super::{
    archive::ExtractOptions,
    global_cache::{install_tool, FindVersion, InstallToolOptions},
    LoadOptions, Tool, Version,
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

    async fn load_with_options(opts: LoadOptions<'_>) -> anyhow::Result<Self> {
        let version = opts.version.unwrap_or(Self::default_version());
        let url = get_download_url(&version)?;
        let extract_opts = ExtractOptions {
            skip_base: true,
            ..Default::default()
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

    fn binary_path(&self) -> &Path {
        self.0.as_path()
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
    use crate::tools::{wasm_bindgen::WasmBindgen, LoadOptions, Tool, ToolExt};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_wasm_bindgen_load_in_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let download_path = temp_dir.path().to_path_buf();
        tokio::fs::create_dir_all(&download_path).await.unwrap();

        let wasm_bingen = WasmBindgen::load_with_options(LoadOptions {
            install_dir: Some(temp_dir.path()),
            ..Default::default()
        })
        .await
        .unwrap();

        let version = wasm_bingen.test_version().unwrap();
        let default_version = WasmBindgen::default_version();
        assert_eq!(version, default_version)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_wasm_bindgen_load() {
        let wasm_bingen = WasmBindgen::load().await.unwrap();

        let version = wasm_bingen.test_version().unwrap();
        let default_version = WasmBindgen::default_version();
        assert_eq!(version, default_version)
    }
}
