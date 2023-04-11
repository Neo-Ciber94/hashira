use clap::Args;
use std::path::PathBuf;

// directories and files included as default in the `public_dir` if not valid is specified.
pub const DEFAULT_INCLUDES: &[&str] = &["public/", "styles/", "favicon.ico"];

#[derive(Args, Debug, Clone)]
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
        help = "Build artifacts in release mode, with optimizations",
        default_value_t = false
    )]
    pub release: bool,

    #[arg(
        long,
        help = "A list of files and directories to copy in the `public_dir`, by default include the `public/`, `styles/` and `favicon.ico` if found"
    )]
    pub include: Vec<PathBuf>,

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
}

impl BuildOptions {
    pub fn resolved_target_dir(&self) -> anyhow::Result<PathBuf> {
        match &self.target_dir {
            Some(path) => Ok(path.clone()),
            None => crate::utils::get_target_dir(),
        }
    }

    pub fn profile_target_dir(&self) -> anyhow::Result<PathBuf> {
        let mut dir = self.resolved_target_dir()?;

        if self.release {
            dir.push("release");
        } else {
            dir.push("debug");
        };

        Ok(dir)
    }
}
