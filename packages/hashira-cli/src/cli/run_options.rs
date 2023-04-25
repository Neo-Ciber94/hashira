use super::{BuildOptions, DevOptions};
use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct RunOptions {
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
}

impl From<&DevOptions> for RunOptions {
    fn from(dev_opts: &DevOptions) -> Self {
        Self {
            build_opts: dev_opts.build_opts.clone(),
            static_dir: dev_opts.static_dir.clone(),
            host: dev_opts.host.clone(),
            port: dev_opts.port,
        }
    }
}
