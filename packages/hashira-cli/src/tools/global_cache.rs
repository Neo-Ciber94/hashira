use std::{collections::HashMap, path::PathBuf, sync::Mutex};

use once_cell::sync::Lazy;
use thiserror::Error;

use crate::tools::{InstallOptions, Installation, Tool};

use super::cache_dir;

static GLOBAL_CACHE: Lazy<Mutex<HashMap<String, PathBuf>>> = Lazy::new(|| Default::default());

#[derive(Debug, Error)]
pub enum GetToolError {
    #[error("{0} was not found in cache")]
    NotFound(String),

    #[error(transparent)]
    FailedDownload(Box<dyn std::error::Error + Send + Sync>),
}

pub struct ToolCache;


impl ToolCache {
    pub async fn get(
        name: &str,
        version: &str,
        download_url: &str,
        installation: Installation,
    ) -> anyhow::Result<PathBuf> {
        match installation {
            Installation::IfRequired => {
                let mut cache = GLOBAL_CACHE.lock().unwrap();
                if let Some(file) = cache.get(name).cloned() {
                    return Ok(file);
                };

                todo!("Download file")
            }
            Installation::Force => todo!(),
            Installation::NoInstall => {
                let mut cache = GLOBAL_CACHE.lock().unwrap();
                if let Some(file) = cache.get(name).cloned() {
                    return Ok(file);
                };

                let file = get_from_cache(name, version)?;
                let file = file.ok_or_else(|| GetToolError::NotFound(name.into()))?;
                cache.insert(name.to_owned(), file.clone());
                Ok(file)
            }
        }
    }
}

fn get_from_cache(name: &str, version: &str) -> anyhow::Result<Option<PathBuf>> {
    let cache_dir = cache_dir()?;
    let file_path = {
        if cfg!(target_os = "windows") {
            PathBuf::from(name).join(format!("{name}-{version}.exe"))
        } else {
            PathBuf::from(name).join(format!("{name}-{version}"))
        }
    };

    match cache_dir.canonicalize(file_path) {
        Ok(file) => Ok(Some(file)),
        Err(err) => {
            tracing::error!("failed to open file: {err}");
            Ok(None)
        }
    }
}
