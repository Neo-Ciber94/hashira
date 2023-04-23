use flate2::read::GzDecoder;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::{Path, PathBuf};
use tar::Archive as TarArchive;
use zip::result::ZipResult;
use zip::ZipArchive;

pub fn decompress_tar_gz<R>(
    reader: &mut R,
    file_name: &str,
    dest: impl AsRef<Path>,
) -> anyhow::Result<PathBuf>
where
    R: Read,
{
    let dest_dir = dest.as_ref();
    anyhow::ensure!(dest_dir.is_dir(), "destination is not a directory");

    let gz_decoder = GzDecoder::new(reader);
    let mut tar_archive = TarArchive::new(gz_decoder);

    let entries = tar_archive.entries()?;
    let mut gz_entry = None;

    for file_result in entries {
        let entry = file_result?;
        let path = entry.path()?;
        let name = path.to_str().unwrap();

        // Find the file to extract
        if name == file_name {
            gz_entry = Some(entry);
            break;
        }
    }

    let Some(mut entry) = gz_entry else {
        anyhow::bail!("Couldn't find file: {file_name}")
    };

    // Create the target directory
    let path = entry.path()?;
    let target_file = dest_dir.join(path);
    if let Some(parent) = target_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Extract and write the file
    let file = std::fs::File::create(&target_file)?;
    let mut buf_writer = BufWriter::new(file);
    let mut buf_reader = BufReader::new(&mut entry);
    std::io::copy(&mut buf_reader, &mut buf_writer)?;
    buf_writer.flush()?;

    Ok(target_file)
}

pub fn decompress_zip<R>(
    reader: &mut R,
    file_name: &str,
    dest: impl AsRef<Path>,
) -> ZipResult<PathBuf>
where
    R: Read + Seek,
{
    let dest_dir = dest.as_ref();
    let mut zip_archive = ZipArchive::new(reader)?;
    let mut zip_file = zip_archive.by_name(file_name)?;

    // Create the target directory
    let zip_path = zip_file.enclosed_name().unwrap();
    let target_file = dest_dir.join(zip_path);
    if let Some(parent) = target_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Extract and write the file
    let file = std::fs::File::create(&target_file)?;
    let mut buf_writer = BufWriter::new(file);
    let mut buf_reader = BufReader::new(&mut zip_file);
    std::io::copy(&mut buf_reader, &mut buf_writer)?;
    buf_writer.flush()?;

    Ok(target_file)
}

// The compressed file.
#[doc(hidden)]
pub struct Compressed(PathBuf);

/// A decompressor for a file.
pub enum Decompressor {
    /// A `tar.gz` decompressor
    TarGz(Compressed),

    /// A `zip` decompressor.
    Zip(Compressed),
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
            _ if path_str.ends_with("tar.gz") => Ok(Some(Decompressor::TarGz(compressed))),
            _ if path_str.ends_with("zip") => Ok(Some(Decompressor::Zip(compressed))),
            _ => {
                anyhow::bail!("no decompressor for: {}", path.display())
            }
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
        }
    }
}
