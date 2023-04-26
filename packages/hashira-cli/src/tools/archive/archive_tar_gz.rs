use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::{Path, PathBuf};

use anyhow::Context;
use flate2::read::GzDecoder;
use tar::{Archive as TarArchive, Entry as TarEntry};

use super::ExtractBehavior;

pub struct ArchiveTarGz {
    tar: Option<Box<TarArchive<GzDecoder<BufReader<File>>>>>,
    should_rewind: bool,
}

impl ArchiveTarGz {
    pub fn new(file: File) -> Self {
        let tar = Box::new(TarArchive::new(GzDecoder::new(BufReader::new(file))));
        ArchiveTarGz {
            tar: Some(tar),
            should_rewind: false,
        }
    }

    fn try_rewind(&mut self) -> anyhow::Result<()> {
        if !self.should_rewind {
            return Ok(());
        };

        let mut archive_file = self.tar.take().unwrap().into_inner().into_inner();

        archive_file
            .rewind()
            .context("error seeking to beginning of archive")?;

        self.tar = Some(Box::new(TarArchive::new(GzDecoder::new(archive_file))));
        self.should_rewind = false;
        Ok(())
    }

    fn find_entry(
        &mut self,
        file: impl AsRef<Path>,
        opts: &ExtractBehavior,
    ) -> anyhow::Result<Option<TarEntry<impl Read>>> {
        self.try_rewind()?;
        let archive = self.tar.as_mut().unwrap();
        self.should_rewind = true;

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
            .with_context(|| format!("`{}` was not found in archive", file.display()))?;

        let out_path = dest.join(file);

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).context("failed creating output directory")?;
        }

        let mut out = File::create(&out_path).context("failed creating output file")?;

        std::io::copy(&mut tar_file, &mut out)
            .context("failed copying over final output file from archive")?;

        if let Ok(mode) = tar_file.header().mode() {
            super::set_file_permissions(&mut out, mode)?;
        }

        Ok(out_path)
    }
}
