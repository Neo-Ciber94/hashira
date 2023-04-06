mod cli;
mod commands;
mod env;
mod utils;
mod pipelines;

use clap::Parser;
use cli::{Cli, Commands};
use env_logger::Env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or(&cli.log_level)).init();

    match cli.command {
        Commands::Build(opts) => commands::build(opts).await,
        Commands::Run(opts) => commands::run(opts).await,
        Commands::Dev(opts) => commands::dev(opts).await,
    }
}
