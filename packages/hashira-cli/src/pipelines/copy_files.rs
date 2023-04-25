use super::{Pipeline, PipelineFile};
use anyhow::Context;
use std::path::{Path, PathBuf};
use tokio::{
    fs::File,
    io::{BufReader, BufWriter},
};

/// A pipeline to copy files, this should be the last pipeline to run
/// because it matches any file.
pub struct CopyFilesPipeline;
impl Pipeline for CopyFilesPipeline {
    fn name(&self) -> &'static str {
        "copy files"
    }

    fn can_process(&self, _: &PipelineFile, _: &std::path::Path) -> bool {
        // We can copy any file
        true
    }

    fn spawn(
        self: Box<Self>,
        files: Vec<PipelineFile>,
        dest_dir: &Path,
    ) -> tokio::task::JoinHandle<anyhow::Result<()>> {
        tokio::spawn(copy_files(files, dest_dir.to_path_buf()))
    }
}

async fn copy_files(files: Vec<PipelineFile>, dest_dir: PathBuf) -> anyhow::Result<()> {
    for target in files {
        let PipelineFile { base_dir, file } = target;
        let dest_path = super::get_file_dest(&base_dir, &file, &dest_dir)?;

        let dest_file = File::create(&dest_path)
            .await
            .context("failed to create destination file")?;

        tracing::debug!("Copying `{}` to `{}`", file.display(), dest_path.display());

        let src_file = File::open(file)
            .await
            .context("failed to open source file")?;

        let mut reader = BufReader::new(src_file);
        let mut writer = BufWriter::new(dest_file);

        tokio::spawn(async move {
            tokio::io::copy(&mut reader, &mut writer)
                .await
                .expect("failed to copy");
        });
    }

    Ok(())
}

