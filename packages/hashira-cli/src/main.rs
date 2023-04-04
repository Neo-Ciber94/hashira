mod cli;
mod commands;

use clap::Parser;
pub use cli::{Cli, Commands};
use env_logger::Env;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            target_dir,
            public_dir,
            static_files,
            dist,
            release,
        } => {
            let args = commands::BuildCommandArgs {
                target_dir,
                public_dir,
                static_files,
                dist,
                release,
            };
            commands::build(args).await?
        }
    }

    Ok(())
}
