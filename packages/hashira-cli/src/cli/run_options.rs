use std::path::PathBuf;
use clap::Args;

#[derive(Args, Debug, Clone)]
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
        help = "Build artifacts in release mode, with optimizations",
        default_value_t = false
    )]
    pub release: bool,

    #[arg(
        long,
        help = "A list of files to copy in the `public_dir` by default include the `public` and `assets` directories, if found"
    )]
    pub include: Vec<String>,

    #[arg(
        long,
        help = "Allow to include files outside the current directory",
        default_value_t = false
    )]
    pub allow_include_external: bool,

    #[arg(
        long,
        help = "Allow to include files inside src/ directory",
        default_value_t = false
    )]
    pub allow_include_src: bool,

    #[arg(
        long,
        default_value_t = false,
        help = "Whether if output the commands output"
    )]
    pub quiet: bool,

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

impl RunOptions {
    pub fn resolved_target_dir(&self) -> anyhow::Result<PathBuf> {
        match &self.target_dir {
            Some(path) => Ok(path.clone()),
            None => crate::utils::get_target_dir(),
        }
    }
}
