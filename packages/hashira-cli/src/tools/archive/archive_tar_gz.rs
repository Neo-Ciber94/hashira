use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::Context;
use flate2::read::GzDecoder;
use tar::{Archive as TarArchive, Entry as TarEntry};

use super::ExtractBehavior;

pub struct ArchiveTarGz(Box<TarArchive<GzDecoder<BufReader<File>>>>);
impl ArchiveTarGz {
    pub fn new(file: File) -> Self {
        Self(Box::new(TarArchive::new(GzDecoder::new(BufReader::new(
            file,
        )))))
    }

    fn find_entry(
        &mut self,
        file: impl AsRef<Path>,
        opts: &ExtractBehavior,
    ) -> anyhow::Result<Option<TarEntry<impl Read>>> {
        let archive = &mut self.0;
        let entries = archive
            .entries()
            .context("failed getting archive entries")?;

        let path = file.as_ref();

        for entry in entries {
            let entry = entry.context("error while getting archive entry")?;
            let name = entry.path().context("invalid entry path")?;
            let mut name = name.components();

            match &opts {
                ExtractBehavior::SkipBasePath => {
                    name.next();
                    if name.as_path() == path {
                        return Ok(Some(entry));
                    }
                }
                ExtractBehavior::Dir(dir) => {
                    let actual_path = dir.join(path);
                    if name.as_path() == actual_path {
                        return Ok(Some(entry));
                    }
                }
                ExtractBehavior::None => {
                    if name.as_path() == path {
                        return Ok(Some(entry));
                    }
                }
            }
        }

        Ok(None)
    }

    pub fn extract_file(
        &mut self,
        file: impl AsRef<Path>,
        dest: &Path,
        opts: &ExtractBehavior,
    ) -> anyhow::Result<PathBuf> {
        let file = file.as_ref();
        let mut tar_file = self
            .find_entry(file, opts)?
            .context("file not found in archive")?;

        let out_path = dest.join(file);

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).context("failed creating output directory")?;
        }

        let mut out = File::create(dest.join(file)).context("failed creating output file")?;

        std::io::copy(&mut tar_file, &mut out)
            .context("failed copying over final output file from archive")?;

        if let Ok(mode) = tar_file.header().mode() {
            super::set_file_permissions(&mut out, mode)?;
        }

        Ok(out_path)
    }
}
