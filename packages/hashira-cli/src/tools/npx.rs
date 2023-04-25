use anyhow::Context;

use super::{node_js::NodeJs, CommandArgs, Tool, ToolExt, Version};
use std::{
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

#[derive(Clone)]
pub struct Npx(PathBuf);

#[allow(dead_code)]
impl Npx {
    pub fn from_node(node: &NodeJs) -> anyhow::Result<Self> {
        let path = node.binary_path();
        let node_dir = path.parent().context("failed to get node directory")?;
        let npx_path = node_dir.join(Self::binary_name());
        Ok(Self(npx_path))
    }

    /// Returns a command used to run the specified package globally
    pub fn exec_global(&self, package: String) -> Command {
        // npx {package}
        let mut args = CommandArgs::new();
        args.arg("install").arg(package);

        self.cmd(args)
    }

    /// Returns a command used to run the specified package in the given directory
    pub fn exec_cmd(&self, package: String, dir: impl AsRef<Path>) -> Command {
        // npx {package} --prefix {dir}
        let mut args = CommandArgs::new();
        args.arg("install")
            .arg(package)
            .arg("--prefix")
            .arg(dir.as_ref());

        self.cmd(args)
    }
}

#[async_trait::async_trait]
impl Tool for Npx {
    fn binary_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "npx.cmd"
        } else {
            "npx"
        }
    }

    fn default_version() -> super::Version {
        Version::new(8, 18, Some(4)) // version of npx for `node 16.20.0`
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
        anyhow::ensure!(
            opts.version.is_none(),
            "cannot specific npx version due this dependant of the `node` version"
        );

        anyhow::ensure!(
            opts.install_dir.is_none(),
            "cannot specific the install location of npx due this dependant of the `node` location"
        );

        let node_js = NodeJs::load().await?;
        let node_path = node_js
            .binary_path()
            .parent()
            .context("failed to get node js directory")?;

        let bin_path = node_path.join(Self::binary_name());
        Ok(Self(bin_path))
    }
}

#[cfg(test)]
mod tests {
    use crate::tools::{node_js::NodeJs, npx::Npx, LoadOptions, Tool, ToolExt, Version};

    #[tokio::test]
    async fn test_npx_from_node() {
        let node_js = NodeJs::load_with_options(LoadOptions {
            version: Some(Version::new(16, 20, Some(0))),
            ..Default::default()
        })
        .await
        .unwrap();

        let npx = Npx::from_node(&node_js).unwrap();
        assert!(npx.test_version().is_ok());
    }
}
