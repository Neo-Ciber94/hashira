use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use super::{archive_tar_gz::ArchiveTarGz, archive_zip::ArchiveZip};

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

    pub fn extract_file(&mut self, file: impl AsRef<Path>, dest: &Path) -> anyhow::Result<PathBuf> {
        anyhow::ensure!(dest.is_dir(), "destination is no a directory");
        anyhow::ensure!(dest.exists(), "destination path don't exists");

        match self {
            Archive::TarGz(tar_gz) => tar_gz.extract_file(file, dest),
            Archive::Zip(zip) => zip.extract_file(file, dest),
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
                super::set_file_permissions(&mut dest_file, 0x755)?; // `rwx` user, `rx` others
                Ok(out_path)
            }
        }
    }
}
