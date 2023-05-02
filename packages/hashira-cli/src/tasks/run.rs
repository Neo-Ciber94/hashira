use super::build::BuildTask;
use crate::cli::{BuildOptions, RunOptions};
use crate::emojis;
use anyhow::Context;
use tokio::sync::Mutex;

use std::path::Path;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::{
    process::{Child, Command},
    sync::broadcast::Sender,
};

// We can only have one running process of the server
static RUNNING_PROCESS: Mutex<Option<Child>> = Mutex::const_new(None);

pub struct RunTask {
    // Options for running the application
    pub(crate) options: Arc<BuildOptions>,

    // Path in the server to serve the static files
    pub(crate) static_dir: String,

    // Host to run the server
    pub(crate) host: String,

    // Port to run the server
    pub(crate) port: u16,

    // A flag indicating if is running in dev mode.
    pub(crate) is_dev: bool,

    // Additional environment variables
    pub(crate) envs: HashMap<&'static str, String>,

    // A receiver for shutdown the executing process
    pub(crate) interrupt_signal: Option<Sender<()>>,

    // Notify when a build is done
    pub(crate) build_done_signal: Option<Sender<()>>,
}

impl RunTask {
    pub fn new(options: RunOptions) -> Self {
        RunTask {
            options: Arc::new(BuildOptions::from(&options)),
            host: options.host,
            port: options.port,
            is_dev: false,
            static_dir: options.static_dir,
            interrupt_signal: None,
            build_done_signal: None,
            envs: Default::default(),
        }
    }

    pub fn env(&mut self, name: &'static str, value: String) {
        self.envs.insert(name, value);
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        // Builds the app
        self.build().await?;

        // Run the generated exe
        self.exec().await?;

        Ok(())
    }

    pub(crate) async fn build(&self) -> anyhow::Result<bool> {
        let build_done_signal = &self.build_done_signal;
        let build_task = BuildTask {
            options: self.options.clone(),
            interrupt_signal: self.interrupt_signal.clone(),
        };

        if !build_task.run_interruptible().await? {
            return Ok(false);
        }

        if let Some(build_done_signal) = build_done_signal {
            if let Err(err) = build_done_signal.send(()) {
                tracing::error!("Error sending build done signal: {err}");
            }
        }

        Ok(true)
    }

    pub(crate) async fn exec(&mut self) -> anyhow::Result<()> {
        self.spawn_server_exec().await.context("failed to run")?;
        Ok(())
    }

    async fn spawn_server_exec(&mut self) -> anyhow::Result<()> {
        if let Some(process) = RUNNING_PROCESS.lock().await.as_mut().take() {
            tracing::debug!("Stopping server...");
            if let Err(err) = process.kill().await {
                tracing::debug!("Failed to kill process: {err}");
            }
        }

        tracing::info!("{}Executing...", emojis::LIGHTING);

        let exec_path = self
            .get_executable()
            .await
            .context("Failed to get executable path")?;

        tracing::debug!("Executable path: {}", exec_path.display());

        let mut cmd = Command::new(exec_path);
        let wasm_lib = crate::utils::get_cargo_lib_name()?;

        // environment variables
        tracing::debug!("host: {}", self.host);
        tracing::debug!("port: {}", self.port);
        tracing::debug!("static files: {}", self.static_dir);

        cmd.env(crate::env::HASHIRA_HOST, &self.host);
        cmd.env(crate::env::HASHIRA_PORT, self.port.to_string());
        cmd.env(crate::env::HASHIRA_STATIC_DIR, &self.static_dir);
        cmd.env(crate::env::HASHIRA_WASM_LIB, wasm_lib);

        for (name, value) in self.envs.iter() {
            cmd.env(name, value);
        }

        let mut child = cmd.spawn()?;

        // If is dev we keep the process, otherwise we just run normally and wait
        if self.is_dev {
            *RUNNING_PROCESS.lock().await = Some(child);
        } else {
            child.wait().await?;
        }

        Ok(())
    }

    async fn get_executable(&self) -> anyhow::Result<PathBuf> {
        let opts = &self.options;
        let exec_name = self.binary_name()?;
        let target_dir = opts.profile_target_dir()?;
        let exec_path = target_dir.join(exec_name);

        // To allow to execute the executable while running we need to create a copy,
        // this is only required if running in `dev`
        if exec_path.exists() && self.is_dev {
            let new_exe_path = append_suffix(&exec_path, "_dev");

            tracing::debug!(
                "Copying executable from: `{}` to `{}`",
                exec_path.display(),
                new_exe_path.display()
            );

            if let Err(err) = tokio::fs::copy(&exec_path, &new_exe_path).await {
                tracing::error!("{err}");
                return Err(err.into());
            }

            return Ok(new_exe_path);
        }

        Ok(exec_path)
    }

    fn binary_name(&self) -> anyhow::Result<String> {
        if cfg!(target_os = "windows") {
            Ok(format!("{}.exe", crate::utils::get_exec_name()?))
        } else {
            crate::utils::get_exec_name()
        }
    }
}

fn append_suffix(path: impl AsRef<Path>, suffix: &str) -> PathBuf {
    let path = path.as_ref();

    if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
        match path.extension().and_then(|s| s.to_str()) {
            Some(ext) => path.with_file_name(format!("{file_name}{suffix}.{ext}")),
            None => path.with_file_name(format!("{file_name}{suffix}")),
        }
    } else {
        path.join(suffix)
    }
}
