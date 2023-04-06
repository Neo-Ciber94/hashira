mod commands;
mod utils;

use clap::{Parser, Subcommand};
use commands::BuildOptions;
use env_logger::Env;

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
    Run,

    #[command(about = "Runs the application in watch mode")]
    Dev,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or(&cli.log_level)).init();

    match cli.command {
        Commands::Build(opts) => commands::build(opts).await,
        Commands::Run => {
            log::info!("running...");
            Ok(())
        }
        Commands::Dev => {
            log::info!("running...");
            Ok(())
        }
    }
}
