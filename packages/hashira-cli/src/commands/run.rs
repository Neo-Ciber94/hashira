use clap::Args;
use std::path::PathBuf;
use tokio::process::Command;
use crate::utils::get_target_dir;
use super::BuildOptions;

#[derive(Args, Debug)]
pub struct RunOptions {
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

    #[arg(
        long,
        default_value_t = false,
        help = "Whether if output the commands output"
    )]
    pub quiet: bool,

    #[arg(
        short,
        long,
        help = "The server path where the static files will be serve",
        default_value = "/static"
    )]
    pub static_dir: String,

    #[arg(
        long,
        help = "The host to run the application",
        default_value = "127.0.0.1"
    )]
    pub host: String,

    #[arg(long, help = "The port to run the application", default_value_t = 5000)]
    pub port: u16,
}

impl RunOptions {
    pub fn resolved_target_dir(&self) -> anyhow::Result<PathBuf> {
        match &self.target_dir {
            Some(path) => Ok(path.clone()),
            None => get_target_dir(),
        }
    }
}

pub async fn run(opts: RunOptions) -> anyhow::Result<()> {
    let build_opts = BuildOptions {
        public_dir: opts.public_dir.clone(),
        target_dir: opts.target_dir.clone(),
        release: opts.release,
        include: opts.include.clone(),
        quiet: opts.quiet,
    };

    super::build_wasm(&build_opts).await?;

    log::info!("Running application");
    cargo_run(&opts).await?;

    log::info!("✅ Done...");
    Ok(())
}

async fn cargo_run(opts: &RunOptions) -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");

    // args
    cmd.arg("run");

    if opts.quiet {
        cmd.arg("--quiet");
    }

    // target dir
    let target_dir = opts.resolved_target_dir()?;
    log::debug!("target dir: {}", target_dir.display());

    cmd.arg("--target-dir");
    cmd.arg(target_dir);

    // release mode?
    if opts.release {
        cmd.arg("--release");
    }

    // environment variables
    log::debug!("host: {}", opts.host);
    log::debug!("port: {}", opts.port);
    log::debug!("static files: {}", opts.static_dir);

    cmd.env(crate::env::HASHIRA_HOST, &opts.host);
    cmd.env(crate::env::HASHIRA_PORT, opts.port.to_string());
    cmd.env(crate::env::HASHIRA_STATIC_DIR, &opts.static_dir);

    // Run
    let mut child = cmd.spawn()?;
    let status = child.wait().await?;
    anyhow::ensure!(status.success(), "failed to run");

    Ok(())
}
