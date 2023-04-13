mod build_options;
mod dev_options;
mod run_options;
mod wasm_opt_level;

pub use build_options::*;
pub use dev_options::*;
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
    #[arg(long, default_value = "info", value_parser = valid_log_level)]
    pub log_level: String,
}

fn valid_log_level(s: &str) -> Result<String, String> {
    const LOG_LEVELS: &[&str] = &["debug", "info", "warn", "error"];

    if LOG_LEVELS.contains(&s) {
        Ok(s.to_owned())
    } else {
        Err(format!(
            "Invalid log level: {s}, expected one of {}",
            LOG_LEVELS.join(", ")
        ))
    }
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
