use std::{path::PathBuf, str::FromStr};
use super::{Tool, Version};

pub struct Sass(PathBuf);

#[async_trait::async_trait]
impl Tool for Sass {
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

    async fn load_with_options(opts: super::LoadOptions<'_>) -> anyhow::Result<Self> {
        todo!()
    }
}
