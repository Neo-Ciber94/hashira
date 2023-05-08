use crate::tools::npm::Npm;

use super::{
    global_cache::{FindVersion, GlobalCache, GlobalCacheError},
    node_js::NodeJs,
    utils::cache_dir,
    Tool, Version,
};
use anyhow::Context;
use std::{path::PathBuf, process::Command, str::FromStr};

// Checkout: https://parceljs.org/

#[derive(Clone)]
pub struct Parcel(PathBuf);

#[async_trait::async_trait]
impl Tool for Parcel {
    fn name() -> &'static str {
        "parcel"
    }

    fn binary_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "parcel.cmd"
        } else {
            "parcel"
        }
    }

    fn default_version() -> super::Version {
        Version::new(2, 8, Some(3))
    }

    fn test_version_args() -> &'static [&'static str] {
        &["--version"]
    }

    fn parse_version(s: &str) -> anyhow::Result<super::Version> {
        let version_text = s.trim();
        Version::from_str(version_text)
    }

    fn binary_path(&self) -> &std::path::Path {
        &self.0
    }

    async fn load_with_options(opts: super::LoadOptions<'_>) -> anyhow::Result<Self> {
        let version = opts.version.unwrap_or(Self::default_version());
        let node_js = NodeJs::load().await.context("failed to get node")?;
        let npm = Npm::from_node(&node_js)?;

        match opts.install_dir {
            Some(dir) => {
                anyhow::ensure!(dir.is_dir(), "`{}` is not a directory", dir.display());
                tokio::fs::create_dir_all(&dir).await?;

                // Install using npm install
                let install_cmd = npm.install_cmd(format!("parcel@{version}"), dir);
                run_silent(install_cmd).context("failed to install parcel")?;

                // The binary is located in {dir}/node_modules/.bin/parcel
                let bin = dir
                    .join("node_modules")
                    .join(".bin")
                    .join(Self::binary_name());
                Ok(Self(bin))
            }
            None => {
                match GlobalCache::find_any::<Self>(FindVersion::Any).await {
                    Ok(bin_path) => Ok(Self(bin_path)),
                    Err(GlobalCacheError::NotFound(_)) => {
                        let dir = cache_dir()?;

                        // Install using npm install
                        let install_cmd = npm.install_cmd(format!("parcel@{version}"), &dir);
                        run_silent(install_cmd).context("failed to install parcel")?;

                        // The binary is located in {dir}/node_modules/.bin/parcel
                        let bin = dir
                            .join("node_modules")
                            .join(".bin")
                            .join(Self::binary_name());
                        Ok(Self(bin))
                    }
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }
        }
    }
}

// Run a command silently and capture the error from its stderr
fn run_silent(mut cmd: Command) -> anyhow::Result<()> {
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;
    let status = child.wait()?;
    let err = {
        child
            .stderr
            .take()
            .and_then(|mut e| std::io::read_to_string(&mut e).ok())
    };

    if status.success() {
        return Ok(());
    }

    if let Some(err) = err {
        anyhow::bail!("{err}")
    } else {
        anyhow::bail!("command failed")
    }
}

#[cfg(test)]
mod tests {
    use crate::tools::{parcel::Parcel, LoadOptions, Tool, ToolExt, Version};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_parcel_version() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path();
        let parcel = Parcel::load_with_options(LoadOptions {
            install_dir: Some(dir),
            ..Default::default()
        })
        .await
        .unwrap();

        let version = parcel.test_version().unwrap();
        assert_eq!(version, Version::new(2, 8, Some(3)));
    }
}
