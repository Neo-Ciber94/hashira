
use anyhow::Context;
use flate2::read::GzDecoder;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::{Path, PathBuf};
use tar::Archive as TarArchive;
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

    let entries: tar::Entries<GzDecoder<&mut R>> = tar_archive
        .entries()
        .context("failed to read tar.gz entries")?;
    let mut gz_entry = None;

    for file_result in entries {
        let entry = file_result.context("failed to extract tar.gz")?;
        let path = entry.path()?;

        if let Some(name) = path.components().last().map(|s| s.as_os_str()) {
            // Find the file to extract
            if name == file_name {
                gz_entry = Some(entry);
                break;
            }
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
    let mut file = std::fs::File::create(&target_file)?;
    {
        let mut buf_writer = BufWriter::new(&mut file);
        let mut buf_reader = BufReader::new(&mut entry);
        std::io::copy(&mut buf_reader, &mut buf_writer)?;
        buf_writer.flush()?;
    }

    // Set the file permissions
    if let Ok(mode) = entry.header().mode() {
        super::set_file_permissions(&mut file, mode)?;
    }

    Ok(target_file)
}

pub fn decompress_zip<R>(
    reader: &mut R,
    file_name: &str,
    dest: impl AsRef<Path>,
) -> anyhow::Result<PathBuf>
where
    R: Read + Seek,
{
    let dest_dir = dest.as_ref();
    let mut zip_archive = ZipArchive::new(reader)?;
    let mut zip_file = zip_archive
        .by_name(file_name)
        .context("failed to find zip entry")?;

    // Create the target directory
    let zip_path = zip_file.enclosed_name().unwrap();
    let target_file = dest_dir.join(zip_path);
    if let Some(parent) = target_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Extract and write the file
    let mut file = std::fs::File::create(&target_file)?;
    {
        let mut buf_writer = BufWriter::new(&mut file);
        let mut buf_reader = BufReader::new(&mut zip_file);
        std::io::copy(&mut buf_reader, &mut buf_writer)?;
        buf_writer.flush()?;
    }

    // Set the file permissions
    if let Some(mode) = zip_file.unix_mode() {
        super::set_file_permissions(&mut file, mode)?;
    }

    Ok(target_file)
}