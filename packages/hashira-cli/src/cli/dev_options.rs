use clap::Args;
use std::path::PathBuf;

use super::BuildOptions;

#[derive(Args, Debug, Clone)]
pub struct DevOptions {
    #[command(flatten)]
    pub build_opts: BuildOptions,

    // ## Options above come from the `BuildOptions` ##
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

    #[arg(long, help = "Path to ignore when looking for changes")]
    pub ignore: Vec<PathBuf>,
}
