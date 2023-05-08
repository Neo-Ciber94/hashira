use super::{
    archive::ExtractOptions,
    global_cache::{install_tool, FindVersion, InstallToolOptions},
    LoadOptions, Tool, Version,
};
use std::{path::PathBuf, str::FromStr};

#[derive(Clone)]
pub struct NodeJs(PathBuf);

#[async_trait::async_trait]
impl Tool for NodeJs {
    fn name() -> &'static str {
        "node"
    }

    fn binary_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "node.exe"
        } else {
            "node"
        }
    }

    fn default_version() -> super::Version {
        Version::new(16, 20, Some(0))
    }

    fn test_version_args() -> &'static [&'static str] {
        &["--version"]
    }

    fn parse_version(s: &str) -> anyhow::Result<super::Version> {
        // Version is on the format v00.00.0, we skip the `v`
        let version_str = &s.trim()[1..];
        Version::from_str(version_str)
    }

    fn binary_path(&self) -> &std::path::Path {
        &self.0
    }

    fn include() -> &'static [&'static str] {
        // Node install in windows the unix and cmd, so we include all,
        if cfg!(target_os = "windows") {
            &["node.exe", "npx", "npx.cmd", "npm", "npm.cmd"]
        } else {
            &["bin/node", "bin/npx", "bin/npm"]
        }
    }

    async fn load_with_options(opts: LoadOptions<'_>) -> anyhow::Result<Self> {
        let version = opts.version.unwrap_or(Self::default_version());
        let url = get_download_url(&version)?;

        let extract_opts = ExtractOptions {
            skip_base: true,
            preserve_dir: false,
        };

        let install_opts = InstallToolOptions {
            dest: opts.install_dir,
            extract_opts,
            find_version: FindVersion::Any,
            min_version: Some(Self::default_version()),
            url: url.as_str(),
        };

        let bin_path = install_tool::<Self>(install_opts).await?;
        Ok(Self(bin_path))
    }
}

fn get_download_url(version: &Version) -> anyhow::Result<String> {
    let os = if cfg!(target_os = "windows") {
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
        anyhow::bail!("unsupported architecture: {os}")
    };

    Ok(match (os, target_arch) {
        ("windows", "x86_64") => {
            format!("https://nodejs.org/download/release/v{version}/node-v{version}-win-x86.zip")
        }
        ("macos", "x86_64") => {
            format!(
                "https://nodejs.org/download/release/v{version}/node-v{version}-darwin-x64.tar.gz"
            )
        }
        ("macos", "aarch64") => {
            format!("https://nodejs.org/download/release/v{version}/node-v{version}-darwin-arm64.tar.gz")
        }
        ("linux", "x86_64") => {
            format!(
                "https://nodejs.org/download/release/v{version}/node-v{version}-linux-x64.tar.gz"
            )
        }
        ("linux", "aarch64") => {
            format!(
                "https://nodejs.org/download/release/v{version}/node-v{version}-linux-arm64.tar.gz"
            )
        }
        _ => anyhow::bail!("unsupported target architecture: {os} {target_arch}"),
    })
}

#[cfg(test)]
mod tests {
    use std::{path::Path, process::Command};

    use crate::tools::{
        archive::{Archive, ExtractOptions},
        node_js::NodeJs,
        LoadOptions, Tool, ToolExt, Version,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn test_load_and_version() {
        let temp_dir = tempfile::tempdir().unwrap();
        let node = NodeJs::load_with_options(LoadOptions {
            install_dir: Some(temp_dir.path()),
            ..Default::default()
        })
        .await
        .unwrap();
        let version = node.test_version().unwrap();
        let default_version = NodeJs::default_version();

        assert_eq!(version, default_version);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_load() {
        let node = NodeJs::load().await.unwrap();
        assert!(node.test_version().is_ok());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_additional_files() {
        let node = NodeJs::load().await.unwrap();

        #[cfg(target_os = "windows")]
        const NPM: &str = "npm.cmd";

        #[cfg(target_os = "windows")]
        const NPX: &str = "npx.cmd";

        #[cfg(not(target_os = "windows"))]
        const NPM: &str = "npm";

        #[cfg(not(target_os = "windows"))]
        const NPX: &str = "npx";

        // The additional files are installed in the same directory
        let dir = node.binary_path().parent().unwrap();

        let npm_output = Command::new(dir.join(NPM))
            .arg("--version")
            .output()
            .unwrap();
        let npm_version = String::from_utf8_lossy(&npm_output.stdout);
        assert!(npm_version.trim().len() > 4); // We just test is no empty

        let npx_output = Command::new(dir.join(NPX))
            .arg("--version")
            .output()
            .unwrap();
        let npx_version = String::from_utf8_lossy(&npx_output.stdout);
        assert!(npx_version.trim().len() > 4); // We just test is no empty
    }

    // Download other versions

    #[tokio::test(flavor = "multi_thread")]
    async fn test_load_and_version_20_0_0() {
        let temp_dir = tempfile::tempdir().unwrap();
        let node = NodeJs::load_with_options(LoadOptions {
            version: Some(Version::new(20, 0, Some(0))),
            install_dir: Some(temp_dir.path()),
        })
        .await
        .unwrap();

        let version = node.test_version().unwrap();
        assert_eq!(version, Version::new(20, 0, Some(0)));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_load_and_version_19_9_0() {
        let temp_dir = tempfile::tempdir().unwrap();
        let node = NodeJs::load_with_options(LoadOptions {
            version: Some(Version::new(19, 9, Some(0))),
            install_dir: Some(temp_dir.path()),
        })
        .await
        .unwrap();

        let version = node.test_version().unwrap();
        assert_eq!(version, Version::new(19, 9, Some(0)));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_load_and_version_18_16_0() {
        let temp_dir = tempfile::tempdir().unwrap();
        let node = NodeJs::load_with_options(LoadOptions {
            version: Some(Version::new(18, 16, Some(0))),
            install_dir: Some(temp_dir.path()),
        })
        .await
        .unwrap();

        let version = node.test_version().unwrap();
        assert_eq!(version, Version::new(18, 16, Some(0)));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_load_and_version_17_9_1() {
        let temp_dir = tempfile::tempdir().unwrap();
        let node = NodeJs::load_with_options(LoadOptions {
            version: Some(Version::new(17, 9, Some(1))),
            install_dir: Some(temp_dir.path()),
        })
        .await
        .unwrap();

        let version = node.test_version().unwrap();
        assert_eq!(version, Version::new(17, 9, Some(1)));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_download_linux_bin() {
        test_download_bin(
            "https://nodejs.org/download/release/v16.20.0/node-v16.20.0-linux-x64.tar.gz",
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_download_macos_bin() {
        test_download_bin(
            "https://nodejs.org/download/release/v16.20.0/node-v16.20.0-darwin-x64.tar.gz",
        )
        .await;
    }

    async fn test_download_bin(url: &str) {
        let temp_dir = tempfile::tempdir().unwrap();
        let downloaded = crate::tools::utils::download_to_dir(url, temp_dir.path())
            .await
            .unwrap();

        let opts = ExtractOptions {
            skip_base: true,
            ..Default::default()
        };

        let mut archive = Archive::new(&downloaded).unwrap();
        let node_js = archive
            .extract_file("bin/node", temp_dir.path(), opts)
            .unwrap();

        let npx = archive
            .extract_file("bin/npx", temp_dir.path(), opts)
            .unwrap();

        let npm = archive
            .extract_file("bin/npm", temp_dir.path(), opts)
            .unwrap();

        assert!(
            node_js.exists(),
            "contents: `{:#?}`",
            debug_contents(&downloaded)
        );

        assert!(
            npx.exists(),
            "contents: `{:#?}`",
            debug_contents(&downloaded)
        );

        assert!(
            npm.exists(),
            "contents: `{:#?}`",
            debug_contents(&downloaded)
        );
    }

    fn debug_contents(path: &Path) -> Vec<String> {
        let mut files = vec![];

        for entry in path.read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                files.push(entry.path().display().to_string());
            }
        }

        files
    }
}
