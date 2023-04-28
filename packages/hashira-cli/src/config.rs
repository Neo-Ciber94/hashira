use crate::cli::BuildOptions;
use std::path::PathBuf;
use tokio::sync::broadcast::Sender;

pub struct Config {
    build: BuildOptions,
    watch: Option<WatchConfig>,
    run: Option<RunConfig>,

    // Signal used to shutdown the processes
    interrupt_signal: Option<Sender<()>>,
}

impl Config {
    pub fn build(&self) -> &BuildOptions {
        &self.build
    }

    pub fn watch(&self) -> &WatchConfig {
        self.watch
            .as_ref()
            .expect("no watch configuration available")
    }

    pub fn run(&self) -> &RunConfig {
        self.run.as_ref().expect("no watch configuration available")
    }

    pub fn interrupt_signal(&self) -> &Sender<()> {
        self.interrupt_signal
            .as_ref()
            .expect("interrupt signal was not set")
    }
}

pub struct RunConfig {
    // Path in the server to serve the static files
    pub static_dir: String,

    // Host to run the server
    pub host: String,

    // Port to run the server
    pub port: u16,
}

pub struct WatchConfig {
    // Host of the reload server
    pub reload_host: String,

    // Port of the reload server
    pub reload_port: u16,

    // Additional paths to watch
    pub watch: Vec<PathBuf>,

    // Paths to ignore while waiting for changes
    pub ignore: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    PreBuild,
    PostBuild,
}

#[derive(Debug, Clone)]
pub struct StageCommand {
    pub command: String,
    pub stage: Stage,
    pub args: Vec<String>,
}
