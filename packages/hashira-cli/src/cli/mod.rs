mod build_options;
mod dev_options;
mod log_level;
mod run_options;
mod wasm_opt_level;

pub use build_options::*;
pub use dev_options::*;
pub use log_level::*;
pub use run_options::*;
pub use wasm_opt_level::*;

//
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[clap(global = true)]
    #[arg(long, default_value = "info")]
    pub log_level: LogLevel,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Build the application")]
    Build(BuildOptions),

    #[command(about = "Build and run the application")]
    Run(RunOptions),

    #[command(about = "Runs the application in watch mode")]
    Dev(DevOptions),
}
