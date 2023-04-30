mod cli;
mod env;
mod pipelines;
mod tasks;
mod tools;
mod utils;

#[allow(dead_code)]
mod config;

use anyhow::Context;
use clap::Parser;
use cli::{Cli, Commands, LogLevel};
use tasks::{build::BuildTask, dev::DevTask, run::RunTask, new::NewTask};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    setup_logger(cli.log_level)?;

    match cli.command {
        Commands::New(opts) => NewTask::new(opts).run().await,
        Commands::Build(opts) => BuildTask::new(opts).run().await,
        Commands::Run(opts) => RunTask::new(opts).run().await,
        Commands::Dev(opts) => DevTask::new(opts).run().await,
    }
}

fn setup_logger(log_level: LogLevel) -> anyhow::Result<()> {
    let env_filter = {
        match log_level {
            LogLevel::Debug => "error,hashira=debug",
            LogLevel::Info => "error,hashira=info",
            LogLevel::Warn => "error,hashira=warn",
            LogLevel::Error => "error,hashira=error",
        }
    };

    if log_level == LogLevel::Debug {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(env_filter))
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_level(true)
                    .compact(),
            )
            .try_init()
            .context("error initializing logging")?
    } else {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(env_filter))
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_level(true)
                    .without_time()
                    .compact(),
            )
            .try_init()
            .context("error initializing logging")?
    };

    Ok(())
}
