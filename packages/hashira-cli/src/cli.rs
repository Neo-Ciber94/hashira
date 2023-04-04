use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(long_about = "build the application")]
    Build {
        #[arg(short, long, help = "the target output directory")]
        target_dir: Option<PathBuf>,

        #[arg(
            short,
            long,
            default_value = "public",
            help = "the directory where the static files will be serve"
        )]
        static_files: PathBuf,

        #[arg(short, long, default_value = "static")]
        public_dir: String,

        #[arg(
            long,
            default_value = "public",
            help = "the directory within the `target_dir` to copy the files"
        )]
        dist: String,

        #[arg(
            short,
            long,
            default_value_t = false,
            help = "whether if this is a release build"
        )]
        release: bool,
    },
}
