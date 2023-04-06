pub mod copy_files;

use std::path::{Path, PathBuf};
use tokio::task::JoinHandle;

/// A pipeline to process files.
pub trait Pipeline {
    /// The name of the pipeline.
    fn name(&self) -> &'static str;

    /// Returns `true` if this pipeline can process the given file.
    fn can_process(&self, file_path: &Path, dest_dir: &Path) -> bool;

    /// Spawn a task that process all the given files.
    fn spawn(self: Box<Self>, files: Vec<PathBuf>, dest_dir: &Path) -> JoinHandle<anyhow::Result<()>>;
}
