use anyhow::Context;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::sync::Mutex;

use super::{archive::ExtractBehavior, Tool, Version};
use crate::tools::{
    archive::Archive,
    utils::{cache_dir, download_to_dir},
};

// A cache for all binaries path
static GLOBAL_CACHE: Lazy<Mutex<HashMap<String, PathBuf>>> = Lazy::new(Default::default);

#[derive(Debug, PartialEq, Eq)]
pub enum FindVersion {
    Exact,
    Any,
}

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
    /// Try get the given tool from cache.
    pub async fn find<T: Tool>() -> Result<PathBuf, GlobalCacheError> {
        let mut cache = GLOBAL_CACHE.lock().await;
        let bin_name = T::binary_name();

        if let Some(bin_path) = cache.get(bin_name).cloned() {
            tracing::debug!("loaded tool: `{bin_name}`");
            return Ok(bin_path);
        }

        let cache_dir = cache_dir().map_err(GlobalCacheError::from_anyhow)?;
        if let Ok(tool_path) = cache_dir.canonicalize(bin_name) {
            cache.insert(bin_name.to_owned(), tool_path.clone());
            return Ok(tool_path);
        }

        Err(GlobalCacheError::NotFound(bin_name.to_owned()))
    }

    /// Try get the given tool from the system and save in cache if can.
    pub async fn find_in_system<T: Tool>(
        opts: FindVersion,
    ) -> Result<(PathBuf, Version), GlobalCacheError> {
        let mut cache = GLOBAL_CACHE.lock().await;
        let bin_name = T::binary_name();

        let Some(bin_path) = which::which(bin_name).ok() else {
            return Err(GlobalCacheError::NotFound(bin_name.to_owned()));
        };

        let version =
            super::unchecked_test_version::<T>(&bin_path).map_err(GlobalCacheError::from_anyhow)?;
        let default_version = T::default_version();
        let match_default_version = version == default_version;

        tracing::debug!("tool loaded from system: `{bin_name} {version}`");
        if opts == FindVersion::Any {
            cache.insert(bin_name.to_owned(), bin_path.clone());
            return Ok((bin_path, version));
        }

        if opts == FindVersion::Exact && match_default_version {
            cache.insert(bin_name.to_owned(), bin_path.clone());
            return Ok((bin_path, version));
        }

        Err(GlobalCacheError::InvalidVersion {
            expected: default_version.to_string(),
            actual: version.to_string(),
        })
    }

    /// Try find the tool in the system and in cache.
    pub async fn find_any<T: Tool>(opts: FindVersion) -> Result<PathBuf, GlobalCacheError> {
        match GlobalCache::find_in_system::<T>(opts).await {
            Ok((bin_path, _)) => Ok(bin_path),
            Err(GlobalCacheError::NotFound(_)) => {
                let bin_path = GlobalCache::find::<T>().await?;
                Ok(bin_path)
            }
            Err(err) => Err(err),
        }
    }

    // Downloads and install the binary and save it in the given directory.
    #[tracing::instrument(level = "debug", skip(opts, dest))]
    pub async fn install<T: Tool>(
        url: &str,
        dest: &Path,
        opts: ExtractBehavior,
    ) -> anyhow::Result<PathBuf> {
        let bin_name = T::binary_name();
        let mut cache = GLOBAL_CACHE.lock().await;

        tracing::info!("⏬ Downloading `{bin_name}`...");

        // Downloads an extract the binary
        let downloaded = download_to_dir(url, dest).await?;
        let mut archive = Archive::new(&downloaded)?;

        let bin_path = archive
            .extract_file(bin_name, dest, opts)
            .with_context(|| format!("failed to extract binary: {bin_name}"))?;

        cache.insert(bin_name.to_owned(), bin_path.clone());

        // Add any additional files
        for additional_file in T::additional_files() {
            let file = archive
                .extract_file(additional_file, dest, opts)
                .with_context(|| format!("failed to extract additional file: {additional_file}"))?;

            cache.insert((*additional_file).to_owned(), file);
        }

        Ok(bin_path)
    }
}
