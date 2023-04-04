use std::{io::ErrorKind, path::PathBuf};

use fs_extra::dir::CopyOptions;
use tokio::process::Command;

pub struct BuildCommandArgs {
    pub target_dir: Option<PathBuf>,
    pub static_files: PathBuf,
    pub dist: String,
    pub public_dir: String,
    pub release: bool,
}

pub async fn build(args: BuildCommandArgs) -> std::io::Result<()> {
    // Create the index.html file
    build_index_html(&args).await?;

    // Create the wasm bundle
    build_wasm(&args).await?;

    // Copy static files to target directory
    copy_static_files(&args).await?;

    // Clean up
    clean_up(&args).await?;

    Ok(())
}

async fn build_index_html(args: &BuildCommandArgs) -> std::io::Result<()> {
    let target_dir = args
        .target_dir
        .to_owned()
        .unwrap_or_else(|| get_out_dir(args.release));
    let mut cmd = Command::new("cargo");

    // Run the main.rs and notify that is a `build`
    cmd.args(["run", "--features", "build"]);

    // Release build
    if args.release {
        cmd.arg("--release");
    }

    // Target directory
    cmd.arg("--target_dir").arg(target_dir);

    // Spawn
    cmd.spawn()?;

    Ok(())
}

async fn build_wasm(args: &BuildCommandArgs) -> std::io::Result<()> {
    let mut target_dir = args
        .target_dir
        .to_owned()
        .unwrap_or_else(|| get_out_dir(args.release));

    let mut cmd = Command::new("trunk");
    cmd.args(["build", "--filehash=false"]);

    // Public dir
    cmd.args(["--public-dir", args.public_dir.as_str()]);

    // Target dir
    target_dir.push(args.dist.to_owned());
    cmd.arg("--dist").arg(target_dir);

    // Spawn
    cmd.spawn()?;

    Ok(())
}

async fn copy_static_files(args: &BuildCommandArgs) -> std::io::Result<()> {
    let mut target_dir = args
        .target_dir
        .to_owned()
        .unwrap_or_else(|| get_out_dir(args.release));

    let static_files = &args.static_files;

    if !static_files.exists() {
        return Ok(());
    }

    target_dir.push(args.dist.to_owned());

    fs_extra::copy_items(&[static_files], target_dir, &CopyOptions::new())
        .map_err(|e| std::io::Error::new(ErrorKind::Other, e))?;

    Ok(())
}

async fn clean_up(_args: &BuildCommandArgs) -> std::io::Result<()> {
    // Remove generated index.html file
    std::fs::remove_file("./index.html")?;

    Ok(())
}

fn get_out_dir(is_release: bool) -> PathBuf {
    let mut target_dir = get_target_dir();

    if is_release {
        target_dir.push("release")
    } else {
        target_dir.push("debug")
    }

    target_dir
}

fn get_target_dir() -> PathBuf {
    let metadata = cargo_metadata::MetadataCommand::new().exec().unwrap();
    metadata.target_directory.as_std_path().to_path_buf()
}
