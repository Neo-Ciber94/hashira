use anyhow::Context;
use directories::ProjectDirs;
use futures::StreamExt;
use reqwest::Client;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

/// Returns the cache directory.
pub fn cache_dir() -> anyhow::Result<PathBuf> {
    let dir = ProjectDirs::from("dev", "hashira-rs", "hashira")
        .context("failed finding project directory")?
        .cache_dir()
        .to_owned();

    std::fs::create_dir_all(&dir).context("failed creating cache directory")?;

    Ok(dir)
}

/// Download a file and write the content to the destination.
pub async fn download<W>(url: &str, dest: &mut W) -> anyhow::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let client = Client::new();
    let res = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to download: {url}"))?;

    let mut stream = res.bytes_stream();
    let mut writer = BufWriter::new(dest);

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.context("failed to download file")?;
        writer
            .write_all(&bytes)
            .await
            .context("failed to write file")?;

        writer.flush().await?;
    }

    Ok(())
}

/// Downloads a file to the give path.
pub async fn download_to_file(url: &str, file_path: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let file_path = file_path.as_ref();

    if let Some(parent) = file_path.parent() {
        anyhow::ensure!(
            parent.exists(),
            "parent directory does not exists: {}",
            parent.display()
        );
    }

    let mut file = tokio::fs::File::create(file_path).await?;
    download(url, &mut file).await?;
    Ok(file_path.to_path_buf())
}

/// Downloads a file to the given directory.
pub async fn download_to_dir(url: &str, target_dir: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    fn get_file_name(url: &str) -> Option<String> {
        url.split('/').last().map(|s| s.to_owned())
    }

    let dir = target_dir.as_ref();
    anyhow::ensure!(dir.is_dir(), "`{}` is not a directory", dir.display());

    let file_name = get_file_name(url)
        .ok_or_else(|| anyhow::anyhow!("unable to get file name from the url: {url}"))?;
    let file_path = dir.join(file_name);
    download_to_file(url, file_path).await
}

#[cfg(test)]
mod test {
    use crate::tools::utils::cache_dir;
    use std::path::Path;

    #[tokio::test]
    async fn test_download() {
        let temp_dir = Path::new("temp/test");
        tokio::fs::create_dir_all(temp_dir).await.unwrap();
        let named_temp = tempfile::NamedTempFile::new_in(temp_dir).unwrap();

        let temp_file = named_temp.path().to_path_buf();
        let mut file = tokio::fs::File::create(&temp_file).await.unwrap();
        super::download(
            "https://raw.githubusercontent.com/Neo-Ciber94/hashira/main/README.md",
            &mut file,
        )
        .await
        .unwrap();

        let content = tokio::fs::read_to_string(&temp_file).await.unwrap();
        assert!(
            content.starts_with("# hashira"),
            "actual contents: `{}`",
            content
        );
    }

    #[tokio::test]
    async fn test_download_to_file() {
        let file_path = Path::new("temp/test/readme_test.md");
        let dest_path = super::download_to_file(
            "https://raw.githubusercontent.com/Neo-Ciber94/hashira/main/README.md",
            file_path,
        )
        .await
        .unwrap();

        let content = tokio::fs::read_to_string(&dest_path).await.unwrap();
        assert!(
            content.starts_with("# hashira"),
            "actual contents: `{}`",
            content
        );
    }

    #[tokio::test]
    async fn test_download_to_dir() {
        let dest = Path::new("temp/test");
        let dest_path = super::download_to_dir(
            "https://raw.githubusercontent.com/Neo-Ciber94/hashira/main/README.md",
            dest,
        )
        .await
        .unwrap();

        assert!(dest_path.ends_with("README.md"));

        let content = tokio::fs::read_to_string(&dest_path).await.unwrap();
        assert!(
            content.starts_with("# hashira"),
            "actual contents: `{}`",
            content
        );
    }

    #[tokio::test]
    async fn test_download_to_cache_dir() {
        let cache_dir = cache_dir().unwrap();
        let temp_dir = tempfile::tempdir_in(cache_dir).unwrap();

        let downloaded = super::download_to_dir(
            "https://github.com/Neo-Ciber94/sample_files/raw/main/file.txt",
            temp_dir.path(),
        )
        .await
        .unwrap();

        let contents = std::fs::read_to_string(&downloaded).unwrap();
        assert_eq!(contents, "Hello World!\n", "actual contents: `{contents}`")
    }

    #[tokio::test]
    async fn test_download_and_tar_gz_archive() {
        let temp_dir = tempfile::tempdir().unwrap();
        let downloaded = super::download_to_dir(
            "https://github.com/Neo-Ciber94/sample_files/raw/main/file.tar.gz",
            temp_dir.path(),
        )
        .await
        .unwrap();

        let downloaded_file = std::fs::File::open(downloaded).unwrap();
        let mut tar_gz = crate::tools::archive::ArchiveTarGz::new(downloaded_file);
        let file_path = tar_gz
            .extract_file("file.txt", temp_dir.path(), &Default::default())
            .unwrap();

        let mut file = std::fs::File::open(file_path).unwrap();
        let contents = std::io::read_to_string(&mut file).unwrap();
        assert_eq!(contents, "Hello World!\n", "actual `{contents}`");
    }

    #[tokio::test]
    async fn test_download_and_zip_archive() {
        let temp_dir = tempfile::tempdir().unwrap();
        let downloaded = super::download_to_dir(
            "https://github.com/Neo-Ciber94/sample_files/raw/main/file.zip",
            temp_dir.path(),
        )
        .await
        .unwrap();

        let downloaded_file = std::fs::File::open(downloaded).unwrap();
        let mut zip = crate::tools::archive::ArchiveZip::new(downloaded_file).unwrap();
        let file_path = zip
            .extract_file("file.txt", temp_dir.path(), &Default::default())
            .unwrap();

        let mut file = std::fs::File::open(file_path).unwrap();
        let contents = std::io::read_to_string(&mut file).unwrap();
        assert_eq!(contents, "Hello World!\n", "actual `{contents}`");
    }

    #[tokio::test]
    async fn test_download_and_none_archive() {
        let temp_dir = tempfile::tempdir().unwrap();
        let downloaded = super::download_to_dir(
            "https://github.com/Neo-Ciber94/sample_files/raw/main/file.txt",
            temp_dir.path(),
        )
        .await
        .unwrap();

        let mut tar_gz = crate::tools::archive::Archive::new(downloaded).unwrap();
        let file_path = tar_gz
            .extract_file("file2.txt", temp_dir.path(), Default::default())
            .unwrap();

        let mut file = std::fs::File::open(file_path).unwrap();
        let contents = std::io::read_to_string(&mut file).unwrap();
        assert_eq!(contents, "Hello World!\n", "actual `{contents}`");
    }
}
