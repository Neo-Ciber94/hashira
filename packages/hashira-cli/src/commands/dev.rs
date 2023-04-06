use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct DevOptions {
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

    #[arg(
        long,
        help = "The host to run the hot reload server",
        default_value = "127.0.0.1"
    )]
    pub reload_host: String,

    #[arg(
        long,
        help = "The port to run the hot reload server",
        default_value_t = 5002
    )]
    pub reload_port: u16,
}

pub async fn dev(_opts: DevOptions) -> anyhow::Result<()> {
    log::info!("hashira dev... (not implemented)");
    Ok(())
}
