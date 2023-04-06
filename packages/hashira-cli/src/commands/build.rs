use anyhow::Context;
use clap::Args;
use glob::glob;
use std::path::Path;
use std::path::PathBuf;
use tokio::process::Command;

use crate::utils::get_cargo_metadata;
use crate::utils::{get_cargo_lib_name, get_target_dir};

#[derive(Args, Debug)]
pub struct BuildOptions {
    #[arg(short, long, help = "Base directory for the artifacts")]
    pub target_dir: Option<PathBuf>,

    #[arg(
        short,
        long,
        help = "Directory relative to the `target_dir` where the static files will be serve from",
        default_value = "public"
    )]
    pub public_dir: PathBuf,

    #[arg(
        long,
        help = "A list of files to copy in the `public_dir` by default include the `public` and `assets` directories, if found"
    )]
    pub include: Vec<String>,

    #[arg(
        short,
        long,
        help = "Build artifacts in release mode, with optimizations",
        default_value_t = false
    )]
    pub release: bool,
}

impl BuildOptions {
    pub fn resolved_target_dir(&self) -> anyhow::Result<PathBuf> {
        let target_dir = match &self.target_dir {
            Some(path) => path.clone(),
            None => get_target_dir(self.release)?,
        };

        Ok(target_dir)
    }
}

pub async fn build(opts: BuildOptions) -> anyhow::Result<()> {
    log::info!("Build started");

    build_server(&opts).await?;
    build_wasm(&opts).await?;
    Ok(())
}

pub async fn build_server(opts: &BuildOptions) -> anyhow::Result<()> {
    log::info!("Building server...");
    cargo_build(opts).await?;

    log::info!("✅ Build server done!");
    Ok(())
}

pub async fn build_wasm(opts: &BuildOptions) -> anyhow::Result<()> {
    log::info!("Building wasm...");
    prepare_public_dir(&opts).await?;

    log::info!("Running cargo build --target wasm32-unknown-unknown...");
    cargo_build_wasm(&opts).await?;

    log::info!("Generating wasm bindings...");
    wasm_bindgen_build(&opts).await?;

    log::info!("Copying files to public directory...");
    include_files(&opts).await?;

    log::info!("✅ Build wasm done!");

    Ok(())
}

async fn prepare_public_dir(opts: &BuildOptions) -> anyhow::Result<()> {
    let mut public_dir = match &opts.target_dir {
        Some(path) => path.clone(),
        None => get_target_dir(opts.release)?,
    };
    public_dir.push(&opts.public_dir);

    if public_dir.exists() {
        log::info!("Preparing public directory...");
        tokio::fs::remove_dir_all(public_dir).await?
    }

    Ok(())
}

async fn cargo_build(opts: &BuildOptions) -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");

    // args
    cmd.arg("build").arg("--quiet");

    // target dir
    let target_dir = opts.resolved_target_dir()?;
    log::debug!("target dir: {}", target_dir.display());

    cmd.arg("--target-dir");
    cmd.arg(target_dir);

    // release mode?
    if opts.release {
        cmd.arg("--release");
    }

    // Run
    let mut child = cmd.spawn()?;
    let status = child.wait().await?;
    anyhow::ensure!(status.success(), "failed to build crate");

    Ok(())
}

async fn cargo_build_wasm(opts: &BuildOptions) -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");

    // args
    cmd.arg("build")
        .args(["--target", "wasm32-unknown-unknown"])
        .arg("--quiet");

    // target dir
    let target_dir = opts.resolved_target_dir()?;
    log::debug!("target dir: {}", target_dir.display());

    cmd.arg("--target-dir");
    cmd.arg(target_dir);

    // release mode?
    if opts.release {
        cmd.arg("--release");
    }

    // Run
    let mut child = cmd.spawn()?;
    let status = child.wait().await?;
    anyhow::ensure!(status.success(), "failed to build wasm crate");

    Ok(())
}

async fn wasm_bindgen_build(opts: &BuildOptions) -> anyhow::Result<()> {
    let mut cmd = Command::new("wasm-bindgen");

    // args
    cmd.args(["--target", "web"]).arg("--no-typescript");

    // out dir
    let mut out_dir = opts.resolved_target_dir()?;
    out_dir.push(&opts.public_dir);
    log::debug!("wasm-bindgen out-dir: {}", out_dir.display());

    cmd.arg("--out-dir").arg(out_dir);

    // wasm to bundle
    // The wasm is located in ${target_dir}/wasm32-unknown-unknown/{mode}/{project_name}.wasm
    let wasm_target_dir = match &opts.target_dir {
        Some(path) => path.clone(),
        None => get_wasm_target_dir(opts.release)?,
    };

    let mut wasm_dir = wasm_target_dir.clone();
    let lib_name = get_cargo_lib_name().context("Failed to get lib name")?;
    wasm_dir.push(format!("{lib_name}.wasm"));
    log::debug!("wasm file dir: {}", wasm_dir.display());

    cmd.arg(wasm_dir);

    // Run
    let mut child = cmd.spawn()?;
    let status = child.wait().await?;
    anyhow::ensure!(status.success(), "failed to build wasm");

    Ok(())
}

struct IncludeFiles {
    glob: String,
    is_default: bool,
}

async fn include_files(opts: &BuildOptions) -> anyhow::Result<()> {
    let mut include_files = Vec::new();

    if opts.include.is_empty() {
        include_files.push(IncludeFiles {
            glob: "public/*".to_string(),
            is_default: true,
        });

        include_files.push(IncludeFiles {
            glob: "assets/*".to_string(),
            is_default: true,
        });

        log::debug!("Copying `public/*` and `assets/*` to public directory");
    } else {
        include_files = opts
            .include
            .clone()
            .into_iter()
            .map(|glob| IncludeFiles {
                glob,
                is_default: false,
            })
            .collect()
    }

    let mut dest_dir = match &opts.target_dir {
        Some(path) => path.clone(),
        None => get_target_dir(opts.release)?,
    };
    dest_dir.push(&opts.public_dir);

    copy_files(&include_files, dest_dir.as_path())
        .await
        .context("Failed to copy files")?;

    Ok(())
}

async fn copy_files(files: &[IncludeFiles], dest_dir: &Path) -> anyhow::Result<()> {
    let mut globs = Vec::new();

    for file in files {
        if !file.is_default {
            let path = Path::new(&file.glob);
            if path.is_dir() {
                anyhow::ensure!(path.exists(), "directory {} does not exist", path.display());
            }
        }

        globs.push(&file.glob);
    }

    let mut files = Vec::new();

    for glob_str in globs {
        for entry in glob(glob_str).expect("Failed to read glob pattern") {
            let path = entry?;
            if path.is_dir() {
                continue;
            }

            log::debug!("Entry to copy: {}", path.display());
            files.push(path);
        }
    }

    if files.is_empty() {
        log::info!("No files to copy");
        return Ok(());
    }

    let mut tasks = Vec::new();

    for file in files {
        let dest_file = dest_dir.join(file.file_name().unwrap());
        log::debug!("Copying `{}` to `{}`", file.display(), dest_file.display());

        let mut reader = tokio::fs::File::open(&file).await?;
        let mut writer = tokio::fs::File::create(dest_file).await?;

        tasks.push(tokio::spawn(async move {
            tokio::io::copy(&mut reader, &mut writer).await?;
            Ok::<(), tokio::io::Error>(())
        }));
    }

    let results = futures::future::join_all(tasks).await;
    for ret in results {
        if let Err(err) = ret {
            log::error!("Failed to copy file: {}", err);
        }
    }

    Ok(())
}

fn get_wasm_target_dir(is_release: bool) -> anyhow::Result<PathBuf> {
    let metadata = get_cargo_metadata()?;
    let mut target_dir = metadata.target_directory.as_std_path().to_path_buf();
    target_dir.push("wasm32-unknown-unknown");

    if is_release {
        target_dir.push("release");
    } else {
        target_dir.push("debug");
    }

    Ok(target_dir)
}
