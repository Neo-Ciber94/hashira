mod archive_tar_gz;
mod archive_zip;

use std::io::{BufReader, BufWriter};
use std::{
    fs::File,
    path::{Path, PathBuf},
};
pub use {archive_tar_gz::*, archive_zip::*};

/// Options for extracting.
#[derive(Debug, Default, Clone, Copy)]
pub struct ExtractOptions {
    /// Skip the base path.
    pub skip_base: bool,

    /// Preserve the path of the files when extracted.
    pub preserve_dir: bool,
}

/// A file that may be compressed.
pub enum Archive {
    TarGz(ArchiveTarGz),
    Zip(ArchiveZip),
    None(File),
}

impl Archive {
    pub fn new(file_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = file_path.as_ref();
        anyhow::ensure!(path.exists(), "file don't exists: {}", path.display());

        let Some(path_str) = path.to_str() else {
            anyhow::bail!("failed to get path: {}", path.display());
        };

        let file = std::fs::File::open(path)?;
        match path_str {
            _ if path_str.ends_with(".tar.gz") => Ok(Self::TarGz(ArchiveTarGz::new(file))),
            _ if path_str.ends_with(".zip") => Ok(Self::Zip(ArchiveZip::new(file)?)),
            _ => Ok(Self::None(file)),
        }
    }

    pub fn extract_file(
        &mut self,
        file: impl AsRef<Path>,
        dest: &Path,
        opts: ExtractOptions,
    ) -> anyhow::Result<PathBuf> {
        anyhow::ensure!(dest.is_dir(), "destination is no a directory");
        anyhow::ensure!(dest.exists(), "destination path don't exists");

        match self {
            Archive::TarGz(tar_gz) => tar_gz.extract_file(file, dest, &opts),
            Archive::Zip(zip) => zip.extract_file(file, dest, &opts),
            Archive::None(src) => {
                let file = file.as_ref();
                let out_path = dest.join(file);

                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let mut dest_file = std::fs::File::create(&out_path)?;
                {
                    let mut reader = BufReader::new(src);
                    let mut writer = BufWriter::new(&mut dest_file);
                    std::io::copy(&mut reader, &mut writer)?;
                }
                set_file_permissions(&mut dest_file, 0x755)?; // `rwx` user, `rx` others
                Ok(out_path)
            }
        }
    }
}

// Sets the file permissions (unix only)
#[cfg_attr(not(unix), allow(unused_variables))]
pub(crate) fn set_file_permissions(file: &mut File, mode: u32) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use anyhow::Context;
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;

        file.set_permissions(Permissions::from_mode(mode))
            .context("failed to set file permissions")?;
    }

    Ok(())
}
