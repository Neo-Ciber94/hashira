use super::Pipeline;
use anyhow::Context;
use std::path::{Path, PathBuf};

/// A pipeline to copy files, this should be the last pipeline to run
/// because it matches any file.
pub struct CopyFilesPipeline;
impl Pipeline for CopyFilesPipeline {
    fn name(&self) -> &'static str {
        "copy files"
    }

    fn can_process(&self, _: &std::path::Path, _: &std::path::Path) -> bool {
        // We can copy any file
        true
    }

    fn spawn(
        self: Box<Self>,
        files: Vec<std::path::PathBuf>,
        dest_dir: &Path,
    ) -> tokio::task::JoinHandle<anyhow::Result<()>> {
        tokio::spawn(copy_files(files, dest_dir.to_path_buf()))
    }
}

async fn copy_files(files: Vec<PathBuf>, dest_dir: PathBuf) -> anyhow::Result<()> {
    for file in files {
        let dest_file = dest_dir.join(file.file_name().unwrap());
        log::debug!("Copying `{}` to `{}`", file.display(), dest_file.display());

        let mut reader = tokio::fs::File::open(&file)
            .await
            .with_context(|| format!("Failed to open file to copy: {}", file.display()))?;

        let mut writer = tokio::fs::File::create(&dest_file).await.with_context(|| {
            format!("Failed to create destination file: {}", dest_file.display())
        })?;

        tokio::spawn(async move {
            tokio::io::copy(&mut reader, &mut writer).await?;
            Ok::<(), tokio::io::Error>(())
        });
    }

    Ok(())
}
