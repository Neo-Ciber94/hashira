use std::{collections::HashMap, path::PathBuf, sync::Mutex};

use once_cell::sync::Lazy;
use thiserror::Error;

use crate::tools::{utils::cache_dir, InstallOptions, Installation, Tool};

static GLOBAL_CACHE: Lazy<Mutex<HashMap<String, PathBuf>>> = Lazy::new(|| Default::default());

/// Cache for the tools.
pub struct GlobalCache;

impl GlobalCache {
    /// Finds the given tool in the cache and return the path to it.
    pub fn get(bin_name: &str) -> anyhow::Result<Option<PathBuf>> {
        let mut cache = GLOBAL_CACHE.lock().unwrap();

        if let Some(bin_path) = cache.get(bin_name).cloned() {
            return Ok(Some(bin_path));
        }

        let cache_dir = cache_dir()?;
        if let Ok(tool_path) = cache_dir.canonicalize(bin_name) {
            cache.insert(bin_name.to_owned(), tool_path.clone());
            return Ok(Some(tool_path));
        }

        Ok(None)
    }

    pub async fn install(bin_name: &str, url: &str) -> anyhow::Result<PathBuf> {
        let mut cache = GLOBAL_CACHE.lock().unwrap();
        
        todo!()
    }
}
