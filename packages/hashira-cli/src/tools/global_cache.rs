use anyhow::Context;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::sync::Mutex;

use super::{archive::ExtractOptions, Tool, Version};
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
            return Ok(bin_path);
        }

        let cache_dir = cache_dir().map_err(GlobalCacheError::from_anyhow)?;
        let bin_path = cache_dir.join(bin_name);

        if !bin_path.exists() {
            return Err(GlobalCacheError::NotFound(bin_name.to_owned()));
        }

        tracing::debug!("loaded tool from cache: {bin_name}");
        cache.insert(bin_name.to_owned(), bin_path.clone());
        Ok(bin_path)
    }

    /// Try get the given tool from the system and save in cache if can.
    pub async fn find_in_system<T: Tool>(
        opts: FindVersion,
    ) -> Result<(PathBuf, Version), GlobalCacheError> {
        T::assert_include_files().map_err(GlobalCacheError::from_anyhow)?;

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
        T::assert_include_files().map_err(GlobalCacheError::from_anyhow)?;

        match GlobalCache::find_in_system::<T>(opts).await {
            Ok((bin_path, _)) => Ok(bin_path),
            Err(GlobalCacheError::NotFound(_)) => {
                let bin_path = GlobalCache::find::<T>().await?;
                Ok(bin_path)
            }
            Err(err) => Err(err),
        }
    }

    // Downloads and cache the binary and save it in the given directory.
    #[tracing::instrument(level = "debug", skip(opts, dest))]
    pub async fn download<T: Tool>(
        url: &str,
        dest: &Path,
        opts: ExtractOptions,
    ) -> anyhow::Result<PathBuf> {
        T::assert_include_files()?;

        let bin_name = T::binary_name();
        let mut cache = GLOBAL_CACHE.lock().await;

        tracing::info!("⏬ Downloading `{name}` from `{url}`...", name = T::name(),);

        // Downloads an extract the binary
        let downloaded = download_to_dir(url, dest).await?;
        let mut archive = Archive::new(&downloaded)?;
        let include_files = T::include();

        // If not include file is declared, we include the binary name
        if include_files.is_empty() {
            let bin_path = archive
                .extract_file(bin_name, dest, opts)
                .with_context(|| format!("failed to extract binary: {bin_name}"))?;

            cache.insert(bin_name.to_owned(), bin_path);
        } else {
            // Add all the required files
            for include_file in T::include() {
                let file = archive
                    .extract_file(include_file, dest, opts)
                    .with_context(|| format!("failed to include file: {include_file}"))?;

                cache.insert((*include_file).to_owned(), file);
            }
        }

        // The binary is within the files added
        let bin_path = dest.join(bin_name);
        anyhow::ensure!(
            bin_path.exists(),
            "`{}` was not found after download",
            bin_path.display()
        );
        Ok(bin_path)
    }
}

#[allow(dead_code)]
pub struct InstallToolOptions;

#[allow(dead_code)]
pub async fn install_tool<T: Tool>(
    url: &str,
    dest: Option<&Path>,
    extract_opts: ExtractOptions,
    find_version: FindVersion,
    min_version: Option<Version>,
) -> anyhow::Result<PathBuf> {
    match dest {
        Some(dir) => {
            anyhow::ensure!(dir.is_dir(), "`{}` is not a directory", dir.display());
            let bin_path = GlobalCache::download::<T>(&url, dir, extract_opts).await?;
            Ok(bin_path)
        }
        None => {
            if let Ok((system_bin, version)) = GlobalCache::find_in_system::<T>(find_version).await
            {
                // minimum version
                if let Some(min_version) = min_version {
                    if version >= min_version {
                        return Ok(system_bin);
                    }
                }
            }

            match GlobalCache::find::<T>().await {
                Ok(bin_path) => Ok(bin_path),
                Err(GlobalCacheError::NotFound(_)) => {
                    // Download and install
                    let cache_path = cache_dir()?;
                    let bin_path =
                        GlobalCache::download::<T>(&url, &cache_path, extract_opts).await?;
                    Ok(bin_path)
                }
                Err(err) => Err(anyhow::anyhow!(err)),
            }
        }
    }
}
