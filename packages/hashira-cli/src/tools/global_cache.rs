use std::{collections::HashMap, ffi::OsStr, path::PathBuf};

use anyhow::Context;
use once_cell::sync::Lazy;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::tools::utils::{cache_dir, download_and_extract};

static GLOBAL_CACHE: Lazy<Mutex<HashMap<String, PathBuf>>> = Lazy::new(Default::default);

#[derive(Debug, Error)]
pub enum GlobalCacheError {
    #[error("the binary {0} was not found")]
    NotFound(String),

    #[error("expected version {expected} but was {actual}")]
    InvalidVersion { expected: String, actual: String },

    #[error(transparent)]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl GlobalCacheError {
    pub fn new(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        GlobalCacheError::Other(error.into())
    }

    pub fn from_anyhow(error: anyhow::Error) -> Self {
        let error = error.to_string();
        Self::new(error)
    }
}

/// Cache for the tools.
pub struct GlobalCache;

impl GlobalCache {
    /// Returns the path of the given binary, if not found search in the cache dir.
    pub async fn find(bin_name: &str) -> Result<PathBuf, GlobalCacheError> {
        let mut cache = GLOBAL_CACHE.lock().await;

        if let Some(bin_path) = cache.get(bin_name).cloned() {
            return Ok(bin_path);
        }

        let cache_dir = cache_dir().map_err(GlobalCacheError::from_anyhow)?;
        if let Ok(tool_path) = cache_dir.canonicalize(bin_name) {
            cache.insert(bin_name.to_owned(), tool_path.clone());
            return Ok(tool_path);
        }

        Err(GlobalCacheError::NotFound(bin_name.to_owned()))
    }

    /// Check and returns the given binary in the system and test if matches the given version,
    /// if so returns it.
    pub async fn find_in_system<I, S>(
        bin_name: &str,
        test_version_args: I,
        expected_version: &str,
    ) -> Result<PathBuf, GlobalCacheError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut cache = GLOBAL_CACHE.lock().await;
        let Some(tool_path) = which::which(bin_name).ok() else {
            return Err(GlobalCacheError::NotFound(bin_name.to_owned()));
        };

        let result = super::utils::exec_and_get_output(&tool_path, test_version_args)
            .map_err(GlobalCacheError::from_anyhow)?;

        if result == expected_version {
            cache.insert(bin_name.to_owned(), tool_path.clone());
            return Ok(tool_path);
        }

        Err(GlobalCacheError::InvalidVersion {
            actual: result,
            expected: expected_version.to_owned(),
        })
    }

    /// Finds the binary in the system or in the cache directory.
    pub async fn find_any<I, S>(
        bin_name: &str,
        test_version_args: I,
        expected_version: &str,
    ) -> Result<PathBuf, GlobalCacheError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        match Self::find(bin_name).await {
            Ok(x) => return Ok(x),
            Err(GlobalCacheError::NotFound(..)) => {}
            Err(err) => return Err(err),
        };

        let system_bin =
            Self::find_in_system(bin_name, test_version_args, expected_version).await?;
        Ok(system_bin)
    }

    /// Downloads and install the binary and save it with the given file name.
    pub async fn install(
        bin_name: &str,
        url: &str,
        file_name: Option<&str>,
        target: Option<PathBuf>,
    ) -> anyhow::Result<PathBuf> {
        let mut cache = GLOBAL_CACHE.lock().await;
        let cache_dir = cache_dir()?;
        let dest = target.unwrap_or(cache_dir.canonicalize(".")?);
        let file_name = file_name.unwrap_or(bin_name);
        let bin_path = download_and_extract(url, file_name, dest)
            .await
            .with_context(|| format!("failed to install: {url}"))?;
        cache.insert(bin_name.to_owned(), bin_path.clone());
        Ok(bin_path)
    }
}
