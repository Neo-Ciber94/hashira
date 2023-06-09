use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use super::ExtractOptions;
use anyhow::Context;
use zip::{read::ZipFile, ZipArchive};

pub struct ArchiveZip(ZipArchive<BufReader<File>>);
impl ArchiveZip {
    pub fn new(file: File) -> anyhow::Result<Self> {
        Ok(Self(ZipArchive::new(BufReader::new(file))?))
    }

    fn find_entry(
        &mut self,
        file: impl AsRef<Path>,
        opts: &ExtractOptions,
    ) -> anyhow::Result<Option<ZipFile>> {
        let archive = &mut self.0;
        let path = file.as_ref();
        let mut idx = None;

        for index in 0..archive.len() {
            let entry = archive
                .by_index(index)
                .context("error while getting archive entry")?;

            let name = entry.enclosed_name().context("invalid entry path")?;
            let mut name = name.components();

            if opts.skip_base {
                name.next();
                if name.as_path() == path {
                    idx = Some(index);
                    break;
                }
            } else if name.as_path() == path {
                idx = Some(index);
                break;
            }
        }

        if let Some(idx) = idx {
            Ok(archive.by_index(idx).ok())
        } else {
            Ok(None)
        }
    }

    pub fn extract_file(
        &mut self,
        file: impl AsRef<Path>,
        dest: &Path,
        opts: &ExtractOptions,
    ) -> anyhow::Result<PathBuf> {
        let file = file.as_ref();

        let Some(mut entry) = self.find_entry(file, opts)? else {
            anyhow::bail!("unable to find {}", file.display())
        };

        let zip_file = entry.enclosed_name().context("invalid entry path")?;
        let mut name = zip_file.components();

        if opts.skip_base {
            name.next();
        }

        let out_path = if opts.preserve_dir {
            dest.join(file)
        } else {
            let file_name = file
                .file_name()
                .map(Path::new)
                .unwrap_or_else(|| panic!("failed to get file name: `{}`", file.display()));

            dest.join(file_name)
        };

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).context("failed to create directory")?;
        }

        let mut out_file = std::fs::File::create(&out_path)?;
        std::io::copy(&mut entry, &mut out_file).context("failed to extract zip file")?;

        if let Some(mode) = entry.unix_mode() {
            super::set_file_permissions(&mut out_file, mode)?
        };

        Ok(out_path)
    }
}
