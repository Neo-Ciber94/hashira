use super::BuildOptions;
use crate::utils::{get_target_dir, interrupt::RUN_INTERRUPT};
use anyhow::Context;
use clap::Args;
use std::{collections::HashMap, path::PathBuf};
use tokio::{
    process::{Child, Command},
    sync::broadcast::Sender,
};

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
            None => get_target_dir(),
        }
    }
}

pub async fn run(opts: RunOptions) -> anyhow::Result<()> {
    run_with_envs(opts, Default::default(), None).await
}

pub(crate) async fn run_with_envs(
    opts: RunOptions,
    additional_envs: HashMap<&'static str, String>,
    build_done_signal: Option<Sender<()>>,
) -> anyhow::Result<()> {
    let build_opts = BuildOptions {
        public_dir: opts.public_dir.clone(),
        target_dir: opts.target_dir.clone(),
        release: opts.release,
        quiet: opts.quiet,
        include: opts.include.clone(),
        allow_include_external: opts.allow_include_external,
        allow_include_src: opts.allow_include_src,
    };

    // Build the wasm and server
    crate::commands::build(build_opts).await?;

    if let Some(build_done_signal) = build_done_signal {
        //let _ = build_done_signal.send(());
        build_done_signal
            .send(())
            .expect("failed to send build done signal");
    }

    // Run the generated exe
    run_server_exec(&opts, additional_envs).await?;

    Ok(())
}

async fn run_server_exec(
    opts: &RunOptions,
    additional_envs: HashMap<&'static str, String>,
) -> anyhow::Result<()> {
    let mut int = RUN_INTERRUPT.subscribe();
    let mut spawn = spawn_server_exec(opts, additional_envs)?;

    tokio::select! {
        status = spawn.wait() => {
            log::debug!("Exited");
            anyhow::ensure!(status?.success(), "failed to run server");
        },
        ret = int.recv() => {
            log::debug!("Interrupt signal received");
            spawn.kill().await?;
            log::debug!("Process killed");

            if let Err(err) = ret {
                log::error!("failed to kill server: {err}");
            }
        }
    }

    log::debug!("Exit run");
    Ok(())
}

fn spawn_server_exec(
    opts: &RunOptions,
    additional_envs: HashMap<&'static str, String>,
) -> anyhow::Result<Child> {
    let exec_path = get_executable_path(&opts).context("Failed to get executable path")?;

    log::debug!("Executable path: {}", exec_path.display());

    let mut cmd = Command::new(exec_path);

    // environment variables
    log::debug!("host: {}", opts.host);
    log::debug!("port: {}", opts.port);
    log::debug!("static files: {}", opts.static_dir);

    cmd.env(crate::env::HASHIRA_HOST, &opts.host);
    cmd.env(crate::env::HASHIRA_PORT, opts.port.to_string());
    cmd.env(crate::env::HASHIRA_STATIC_DIR, &opts.static_dir);

    for (name, value) in additional_envs {
        cmd.env(name, value);
    }

    let child = cmd.spawn()?;
    Ok(child)
}

fn get_executable_path(opts: &RunOptions) -> anyhow::Result<PathBuf> {
    let exec_name = crate::utils::get_exec_name()?;
    let mut target_dir = opts.resolved_target_dir()?;

    if opts.release {
        target_dir.push("release");
    } else {
        target_dir.push("debug");
    }

    let exec_path = target_dir.join(format!("{exec_name}.exe"));
    Ok(exec_path)
}
