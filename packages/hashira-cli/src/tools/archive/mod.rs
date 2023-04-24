#[allow(clippy::module_inception)]
mod archive;
mod archive_tar_gz;
mod archive_zip;
mod helpers;

use std::{path::{Path, PathBuf}, fs::File};

pub use helpers::*;
pub use {archive::*, archive_tar_gz::*, archive_zip::*};

// The compressed file.
#[doc(hidden)]
pub struct Compressed(PathBuf);

/// A decompressor for a file.
pub enum Decompressor {
    /// A `tar.gz` decompressor
    TarGz(Compressed),

    /// A `zip` decompressor.
    Zip(Compressed),

    /// No decompression just copies the files.
    Copy(Compressed),
}

impl Decompressor {
    /// Gets a decompressor for the given path.
    /// if not extension is found, will return a decompressor that just copies the contents.
    ///
    /// # Params
    /// - path: The path of the file to decompress
    pub fn get(file_path: impl AsRef<Path>) -> anyhow::Result<Option<Self>> {
        let path = file_path.as_ref();
        anyhow::ensure!(path.exists(), "file don't exists: {}", path.display());

        let Some(path_str) = path.to_str() else {
            anyhow::bail!("failed to get path: {}", path.display());
        };

        let compressed = Compressed(path.to_path_buf());
        match path_str {
            _ if path_str.ends_with(".tar.gz") => Ok(Some(Decompressor::TarGz(compressed))),
            _ if path_str.ends_with(".zip") => Ok(Some(Decompressor::Zip(compressed))),
            _ if path_str.contains('.') => {
                anyhow::bail!("no decompressor for: {}", path.display())
            }
            _ => Ok(Some(Decompressor::Copy(compressed))),
        }
    }

    /// Extracts the file with the given name to the given destination path.
    pub fn extract_file(&self, file_name: &str, dest: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
        match self {
            Decompressor::TarGz(Compressed(f)) => {
                let mut reader = std::fs::File::open(f)?;
                let file = decompress_tar_gz(&mut reader, file_name, dest)?;
                Ok(file)
            }
            Decompressor::Zip(Compressed(f)) => {
                let mut reader = std::fs::File::open(f)?;
                let file = decompress_zip(&mut reader, file_name, dest)?;
                Ok(file)
            }
            Decompressor::Copy(Compressed(f)) => {
                let dest_dir = dest.as_ref();
                std::fs::create_dir_all(dest_dir)?;
                let file_path = dest_dir.join(file_name);

                let mut reader = std::fs::File::open(f)?;
                let mut writer = std::fs::File::create(&file_path)?;
                std::io::copy(&mut reader, &mut writer)?;
                set_file_permissions(&mut writer, 0x755)?;
                Ok(file_path)
            }
        }
    }
}

// Sets the file permissions (unix only)
#[cfg_attr(not(unix), allow(unused_variables))]
pub(crate) fn set_file_permissions(file: &mut File, mode: u32) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;

        file.set_permissions(Permissions::from_mode(mode))
            .context("failed to set file permissions")?;
    }

    Ok(())
}
