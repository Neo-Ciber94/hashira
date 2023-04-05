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
    build_client(&args).await?;

    // Create the wasm bundle
    build_wasm(&args).await?;

    // Copy static files to target directory
    copy_static_files(&args).await?;

    Ok(())
}

async fn build_client(args: &BuildCommandArgs) -> std::io::Result<()> {
    let target_dir = args
        .target_dir
        .to_owned()
        .unwrap_or_else(|| get_out_dir(args.release));

    // cargo build
    let mut cmd = Command::new("cargo");
    cmd.args(["build"]);

    // Release build?
    if args.release {
        cmd.arg("--release");
    }

    // Target wasm
    cmd.args(["--target", "wasm32-unknown-unknown"]);

    // Target directory
    cmd.arg("--target-dir").arg(target_dir);

    // Spawn
    cmd.spawn()?;

    Ok(())
}

async fn build_wasm(args: &BuildCommandArgs) -> std::io::Result<()> {
    let mut target_dir = args
        .target_dir
        .to_owned()
        .unwrap_or_else(|| get_out_dir(args.release));

    let mut cmd = Command::new("wasm-bindgen");
    cmd.args(["--target", "web"]);
    cmd.arg("--no-typescript");

    // Public dir
    cmd.args(["--out-dir", args.public_dir.as_str()]);

    // Target dir
    target_dir.push("wasm32-unknown-unknown");

    if args.release {
        target_dir.push("release");
    } else {
        target_dir.push("debug");
    }

    cmd.arg(target_dir);

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

fn get_out_dir(is_release: bool) -> PathBuf {
    let mut target_dir = get_target_dir();

    if is_release {
        target_dir.push("release")
    } else {
        target_dir.push("debug")
    }

    target_dir
}

fn get_cargo_metadata() -> cargo_metadata::Metadata {
    let metadata = cargo_metadata::MetadataCommand::new().exec().unwrap();
    metadata
}

fn get_target_dir() -> PathBuf {
    let cargo_metadata = get_cargo_metadata();
    cargo_metadata.target_directory.as_std_path().to_path_buf()
}


