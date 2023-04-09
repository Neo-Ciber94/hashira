use crate::cli::BuildOptions;
use crate::pipelines::{copy_files::CopyFilesPipeline, Pipeline};
use anyhow::Context;
use std::path::Path;
use tokio::process::{Child, Command};
use tokio::sync::broadcast::Sender;

struct IncludeFiles {
    glob: String,
    is_default: bool,
}

// directories and files included as default in the `public_dir` if not valid is specified.
const DEFAULT_INCLUDES: &[&str] = &["public/*", "favicon.ico"];

pub struct BuildTask {
    // Options used to build the project
    pub options: BuildOptions,

    // A receiver for shutdown the executing process
    pub shutdown_signal: Option<Sender<()>>,
}

impl BuildTask {
    pub fn new(options: BuildOptions) -> Self {
        BuildTask {
            options,
            shutdown_signal: None,
        }
    }

    /// Runs the build operation
    pub async fn run(self) -> anyhow::Result<()> {
        log::info!("Build started");

        self.build_server().await?;
        self.build_wasm().await?;
        Ok(())
    }

    /// Builds the server
    pub async fn build_server(&self) -> anyhow::Result<()> {
        log::info!("Building server...");
        self.cargo_build().await?;

        log::info!("✅ Build server done!");
        Ok(())
    }

    /// Builds the wasm bundle
    pub async fn build_wasm(&self) -> anyhow::Result<()> {
        log::info!("Building wasm...");
        self.prepare_public_dir().await?;

        log::info!("Running cargo build --target wasm32-unknown-unknown...");
        self.cargo_build_wasm().await?;

        log::info!("Generating wasm bindings...");
        self.wasm_bindgen().await?;

        log::info!("Copying files to public directory...");
        self.include_files().await?;

        log::info!("✅ Build wasm done!");

        Ok(())
    }

    async fn prepare_public_dir(&self) -> anyhow::Result<()> {
        let opts = &self.options;
        let mut public_dir = opts.resolved_target_dir()?;

        if opts.release {
            public_dir.push("release");
        } else {
            public_dir.push("debug");
        }

        public_dir.push(&opts.public_dir);
        log::info!("Preparing public directory: {}", public_dir.display());

        anyhow::ensure!(public_dir.exists(), "Public directory was not found on");

        tokio::fs::remove_dir_all(&public_dir)
            .await
            .with_context(|| format!("failed to remove dir: {}", public_dir.display()))?;

        Ok(())
    }

    async fn cargo_build_wasm(&self) -> anyhow::Result<()> {
        let mut spawn = self.spawn_cargo_build_wasm()?;

        let Some(shutdown_signal) = &self.shutdown_signal else {
            let status = spawn.wait().await?;
            anyhow::ensure!(status.success(), "failed to build wasm crate");
            return Ok(());
        };

        let mut int = shutdown_signal.subscribe();
        tokio::select! {
            status = spawn.wait() => {
                log::debug!("Exited: {status:?}");
            },
            ret = int.recv() => {
                spawn.kill().await?;

                if let Err(err) = ret {
                    log::error!("failed to kill build wasm task: {err}");
                }
            }
        }

        Ok(())
    }

    async fn wasm_bindgen(&self) -> anyhow::Result<()> {
        let mut spawn = self.spawn_wasm_bindgen()?;

        let Some(shutdown_signal) = &self.shutdown_signal else {
            let status = spawn.wait().await?;
            anyhow::ensure!(status.success(), "failed to build wasm crate");
            return Ok(());
        };

        let mut int = shutdown_signal.subscribe();

        tokio::select! {
            status = spawn.wait() => {
                log::debug!("Exited: {status:?}");
            },
            ret = int.recv() => {
                spawn.kill().await?;

                if let Err(err) = ret {
                    log::error!("failed to kill build wasm-bingen task: {err}");
                }
            }
        }

        Ok(())
    }

    async fn cargo_build(&self) -> anyhow::Result<()> {
        let mut spawn = self.spawn_cargo_build()?;

        let Some(shutdown_signal) = &self.shutdown_signal else {
            let status = spawn.wait().await?;
            anyhow::ensure!(status.success(), "failed to build server");
            return Ok(());
        };

        let mut int = shutdown_signal.subscribe();

        tokio::select! {
            status = spawn.wait() => {
                log::debug!("Exited: {status:?}");
            },
            ret = int.recv() => {
                spawn.kill().await?;

                if let Err(err) = ret {
                    log::error!("failed to kill cargo build task: {err}");
                }
            }
        }

        Ok(())
    }

    async fn include_files(&self) -> anyhow::Result<()> {
        let opts = &self.options;
        let include_files: Vec<IncludeFiles>;

        if opts.include.is_empty() {
            include_files = DEFAULT_INCLUDES
                .into_iter()
                .map(|s| (*s).to_owned())
                .map(|glob| IncludeFiles {
                    glob,
                    is_default: true,
                })
                .collect::<Vec<_>>();
        } else {
            include_files = opts
                .include
                .clone()
                .into_iter()
                .map(|glob| IncludeFiles {
                    glob,
                    is_default: false,
                })
                .collect::<Vec<_>>();
        }

        let mut dest_dir = opts.resolved_target_dir()?.join({
            if opts.release {
                "release"
            } else {
                "debug"
            }
        });

        dest_dir.push(&opts.public_dir);

        process_files(&include_files, dest_dir.as_path(), opts)
            .await
            .context("Failed to copy files")?;

        Ok(())
    }

    fn spawn_cargo_build_wasm(&self) -> anyhow::Result<Child> {
        let opts = &self.options;
        let mut cmd = Command::new("cargo");

        // args
        cmd.arg("build")
            .args(["--target", "wasm32-unknown-unknown"]);

        if opts.quiet {
            cmd.arg("--quiet");
        }

        // target dir
        let target_dir = opts.resolved_target_dir()?;
        log::debug!("target dir: {}", target_dir.display());

        cmd.arg("--target-dir");
        cmd.arg(target_dir);

        // release mode?
        if opts.release {
            cmd.arg("--release");
        }

        // Run
        let child = cmd.spawn()?;
        Ok(child)
    }

    fn spawn_cargo_build(&self) -> anyhow::Result<Child> {
        let opts = &self.options;
        let mut cmd = Command::new("cargo");

        // args
        cmd.arg("build");

        if opts.quiet {
            cmd.arg("--quiet");
        }

        // target dir
        let target_dir = opts.resolved_target_dir()?;
        log::debug!("target dir: {}", target_dir.display());

        cmd.arg("--target-dir");
        cmd.arg(target_dir);

        // release mode?
        if opts.release {
            cmd.arg("--release");
        }

        // Run
        let child = cmd.spawn()?;
        Ok(child)
    }

    fn spawn_wasm_bindgen(&self) -> anyhow::Result<Child> {
        let opts = &self.options;

        // TODO: Download wasm-bindgen if doesn't exists on the machine
        let mut cmd = Command::new("wasm-bindgen");

        // args
        cmd.args(["--target", "web"]).arg("--no-typescript");

        // out dir
        let mut out_dir = opts.resolved_target_dir()?.join({
            if opts.release {
                "release"
            } else {
                "debug"
            }
        });

        out_dir.push(&opts.public_dir);
        log::debug!("wasm-bindgen out-dir: {}", out_dir.display());

        cmd.arg("--out-dir").arg(out_dir);

        // wasm to bundle
        // The wasm is located in ${target_dir}/wasm32-unknown-unknown/{profile}/{project_name}.wasm
        let wasm_target_dir = opts.resolved_target_dir()?.join({
            if opts.release {
                "wasm32-unknown-unknown/release"
            } else {
                "wasm32-unknown-unknown/debug"
            }
        });

        let mut wasm_dir = wasm_target_dir.clone();
        let lib_name = crate::utils::get_cargo_lib_name().context("Failed to get lib name")?;
        wasm_dir.push(format!("{lib_name}.wasm"));
        log::debug!("wasm file dir: {}", wasm_dir.display());

        cmd.arg(wasm_dir);

        // Run
        let child = cmd.spawn()?;
        Ok(child)
    }
}

fn is_outside_directory(base: &Path, path: &Path) -> anyhow::Result<bool> {
    let base_dir = base.canonicalize()?;
    let path_dir = path.canonicalize()?;

    match path_dir.strip_prefix(base_dir) {
        Ok(_) => Ok(false),
        Err(_) => Ok(true),
    }
}

fn is_inside_src(base: &Path, path: &Path) -> anyhow::Result<bool> {
    if !base.join("src").exists() {
        log::debug!("`src` directory not found");
        return Ok(false);
    }

    let base_dir = base.canonicalize()?;
    let path_dir = path.canonicalize()?;

    match path_dir.strip_prefix(base_dir) {
        Ok(remaining) => {
            if remaining.starts_with("src") {
                return Ok(true);
            }

            Ok(false)
        }
        Err(_) => Ok(false),
    }
}

async fn process_files(
    files: &[IncludeFiles],
    dest_dir: &Path,
    opts: &BuildOptions,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir().context("failed to get current working directory")?;
    let mut globs = Vec::new();

    for file in files {
        if !file.is_default {
            let path = Path::new(&file.glob);
            if path.is_dir() {
                anyhow::ensure!(path.exists(), "directory {} does not exist", path.display());
            }
        }

        globs.push(&file.glob);
    }

    let mut files = Vec::new();

    for glob_str in globs {
        for entry in glob::glob(glob_str).expect("Failed to read glob pattern") {
            let path = entry?;

            // We ignore directories
            if path.is_dir() {
                continue;
            }

            if !opts.allow_include_external && is_outside_directory(&cwd, &path)? {
                log::error!("{} is outside {}", path.display(), cwd.display());

                anyhow::bail!(
                    "Path to include cannot be outside the current directory, use `--allow-include-external` to include files outside the current directory"
                );
            }

            if !opts.allow_include_src && is_inside_src(&cwd, &path)? {
                log::error!("{} is inside `src` directory", path.display());

                anyhow::bail!(
                    "Path to include cannot be inside the src directory, use `--allow-include-src` to include files inside the src directory"
                );
            }

            log::debug!("Entry: {}", path.display());
            files.push(path);
        }
    }

    if files.is_empty() {
        log::info!("No files to process");
        return Ok(());
    }

    let mut pipelines = get_pipelines();
    let mut tasks = Vec::new();

    loop {
        if files.is_empty() {
            if !pipelines.is_empty() {
                let pipeline_names = pipelines.iter().map(|p| p.name()).collect::<Vec<_>>();
                log::info!(
                    "No more files to process, the next pipelines were not run: {}",
                    pipeline_names.join(", ")
                );
            }

            break;
        }

        let Some(pipeline) = pipelines.pop() else {
            break;
        };

        let mut target_files = vec![];
        let mut i = 0;

        while i < files.len() {
            if pipeline.can_process(&files[i], dest_dir) {
                let file = files.remove(i);
                target_files.push(file);
            } else {
                i += 1;
            }
        }

        tasks.push(async {
            let pipeline_name = pipeline.name().to_owned();
            pipeline
                .spawn(target_files, dest_dir)
                .await
                .with_context(|| format!("error processing `{pipeline_name}` pipeline"))
        });
    }

    let results = futures::future::join_all(tasks).await;
    for ret in results {
        if let Err(err) = ret {
            log::error!("{err}");
        }
    }

    Ok(())
}

// TODO: Should we just process the pipeline in order and forget about using a Box<dyn Pipeline>?
fn get_pipelines() -> Vec<Box<dyn Pipeline + Send>> {
    vec![
        // TODO: Add pipeline to process SCSS, SASS

        // Add any additional pipelines, all should be place before copy
        Box::new(CopyFilesPipeline),
    ]
}
