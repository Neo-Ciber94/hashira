mod cli;
mod commands;
mod env;
mod pipelines;
mod tasks;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use env_logger::Env;
use tasks::{build_task::BuildTask, run_task::RunTask, dev_task::DevTask};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or(&cli.log_level)).init();

    match cli.command {
        Commands::Build(opts) => BuildTask::new(opts).run().await,
        Commands::Run(opts) => RunTask::new(opts).run().await,
        Commands::Dev(opts) => DevTask::new(opts).run().await,
    }
}
