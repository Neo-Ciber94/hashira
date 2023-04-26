use anyhow::Context;

use super::{node_js::NodeJs, Tool, ToolExt, Version};
use std::{
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

#[derive(Clone)]
pub struct Npm(PathBuf);

impl Npm {
    pub fn from_node(node: &NodeJs) -> anyhow::Result<Self> {
        let path = node.binary_path();
        let node_dir = path.parent().context("failed to get node directory")?;
        let npm_path = node_dir.join(Self::binary_name());
        Ok(Self(npm_path))
    }

    /// Returns a command used to install the specified package in the given directory
    pub fn install_cmd(&self, package: String, dir: impl AsRef<Path>) -> Command {
        // Install using npm install {package} --prefix {dir}
        let mut cmd = self.cmd();
        cmd.arg("install")
            .arg(package)
            .arg("--prefix")
            .arg(dir.as_ref());

        cmd
    }
}

#[async_trait::async_trait]
impl Tool for Npm {
    fn binary_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "npm.cmd"
        } else {
            "npm"
        }
    }

    fn default_version() -> super::Version {
        Version::new(8, 18, Some(4)) // version of npm for `node 16.20.0`
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
            "cannot specific npm version due this dependant of the `node` version"
        );

        anyhow::ensure!(
            opts.install_dir.is_none(),
            "cannot specific the install location of npm due this dependant of the `node` location"
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
    use crate::tools::{node_js::NodeJs, npm::Npm, LoadOptions, Tool, ToolExt, Version};

    #[tokio::test]
    async fn test_npm_from_node() {
        let node_js = NodeJs::load_with_options(LoadOptions {
            version: Some(Version::new(16, 20, Some(0))),
            ..Default::default()
        })
        .await
        .unwrap();

        let npm = Npm::from_node(&node_js).unwrap();
        assert!(npm.test_version().is_ok());
    }
}
