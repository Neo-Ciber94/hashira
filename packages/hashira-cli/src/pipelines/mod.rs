pub mod copy_files;

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


/// Computes the file destination within the `dest_dir` base on his base directory.
///
/// # Remarks
/// All the input should be canonized before computing the paths.
///
/// # Example
/// - If the destination path is: `public/`
/// - The file is `products.json`
/// - And the file_dir is `assets/json`
///
/// The destination will be: `public/assets/json`
pub fn get_file_dest(file_dir: &Path, file: &Path, dest_dir: &Path) -> anyhow::Result<PathBuf> {
    let file_name = file.file_name().unwrap();

    match file.strip_prefix(&file_dir) {
        Ok(relative) => {
            let dir = dest_dir.join(relative.parent().unwrap());
            std::fs::create_dir_all(&dir)?;
            Ok(dir.join(file_name))
        }
        Err(_) => Ok(dest_dir.join(file_name)),
    }
}
