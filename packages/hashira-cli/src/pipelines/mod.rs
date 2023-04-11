pub mod copy_files;
pub mod css;

use std::path::{Path, PathBuf};
use tokio::task::JoinHandle;

/// Represents a file to process in a pipeline.
#[derive(Debug)]
pub struct PipelineFile {
    /// The base directory of the file, which can be used to calculate the relative path.
    pub base_dir: PathBuf,

    /// The actual file to process.
    pub file: PathBuf,
}

/// A pipeline to process files.
pub trait Pipeline {
    /// The name of the pipeline.
    fn name(&self) -> &'static str;

    /// Returns `true` if this pipeline can process the given file.
    fn can_process(&self, src: &PipelineFile, dest_dir: &Path) -> bool;

    /// Spawn a task that process all the given files.
    fn spawn(
        self: Box<Self>,
        files: Vec<PipelineFile>,
        dest_dir: &Path,
    ) -> JoinHandle<anyhow::Result<()>>;
}
