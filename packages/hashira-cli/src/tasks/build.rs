use crate::cli::{BuildOptions, WasmOptimizationLevel, DEFAULT_INCLUDES};
use crate::pipelines::PipelineFile;
use crate::pipelines::{copy_files::CopyFilesPipeline, Pipeline};
use crate::tools::sass::Sass;
use crate::tools::wasm_bindgen::WasmBindgen;
use crate::tools::{Tool, ToolExt};
use crate::utils::wait_interruptible;
use anyhow::Context;
use lightningcss::stylesheet::PrinterOptions;
use lightningcss::{
    bundler::{Bundler, FileProvider},
    stylesheet::ParserOptions,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::BufReader;
use tokio::process::{Child, Command};
use tokio::sync::broadcast::Sender;
use wasm_opt::OptimizationOptions;
#[derive(Debug)]
struct IncludeFiles {
    path: PathBuf,
    is_default: bool,
}

pub struct BuildTask {
    // Options used to build the project
    pub options: Arc<BuildOptions>,

    // A receiver for shutdown the executing process
    pub interrupt_signal: Option<Sender<()>>,
}

impl BuildTask {
    pub fn new(options: BuildOptions) -> Self {
        BuildTask {
            options: Arc::new(options),
            interrupt_signal: None,
        }
    }

    /// Runs the build operation
    pub async fn run(self) -> anyhow::Result<()> {
        self.build_server().await?;
        self.build_wasm().await?;
        Ok(())
    }

    /// Builds the server
    pub async fn build_server(&self) -> anyhow::Result<()> {
        tracing::info!("📦 Building server...");
        self.cargo_build().await?;

        tracing::info!("✅ Server build done!");
        Ok(())
    }

    /// Builds the wasm bundle
    pub async fn build_wasm(&self) -> anyhow::Result<()> {
        // Cleanup the public dir
        self.prepare_public_dir().await?;

        // Start Wasm build
        tracing::info!("📦 Building Wasm...");

        self.cargo_build_wasm().await?;
        self.wasm_bindgen().await?;
        self.optimize_wasm().await?; // If the optimization flag is set or in release mode
        self.include_files().await?;

        tracing::info!("✅ Wasm build done!");
        Ok(())
    }

    async fn prepare_public_dir(&self) -> anyhow::Result<()> {
        let opts = &self.options;
        let mut public_dir = opts.profile_target_dir()?;

        public_dir.push(&opts.public_dir);
        tracing::info!("🚧 Preparing public directory: {}", public_dir.display());

        if public_dir.exists() {
            tokio::fs::remove_dir_all(&public_dir)
                .await
                .with_context(|| format!("failed to remove dir: {}", public_dir.display()))?;
        }

        tokio::fs::create_dir_all(public_dir)
            .await
            .context("failed to create public directory")?;

        Ok(())
    }

    async fn cargo_build_wasm(&self) -> anyhow::Result<()> {
        tracing::debug!("Running cargo build --target wasm32-unknown-unknown...");

        let spawn = self.spawn_cargo_build_wasm()?;
        wait_interruptible(spawn, self.interrupt_signal.clone())
            .await
            .context("cargo build wasm failed")?;
        Ok(())
    }

    async fn wasm_bindgen(&self) -> anyhow::Result<()> {
        tracing::debug!("Generating wasm bindings...");

        let spawn = self.spawn_wasm_bindgen().await?;
        wait_interruptible(spawn, self.interrupt_signal.clone())
            .await
            .context("failed to run wasm-bindgen")?;
        Ok(())
    }

    async fn cargo_build(&self) -> anyhow::Result<()> {
        let spawn = self.spawn_cargo_build()?;
        wait_interruptible(spawn, self.interrupt_signal.clone())
            .await
            .context("cargo build failed")?;
        Ok(())
    }

    async fn include_files(&self) -> anyhow::Result<()> {
        tracing::info!("📂 Copying files to public directory...");

        let opts = &self.options;

        let include_files = if opts.include.is_empty() {
            DEFAULT_INCLUDES
                .iter()
                .map(PathBuf::from)
                .map(|path| IncludeFiles {
                    path,
                    is_default: true,
                })
                .collect::<Vec<_>>()
        } else {
            opts.include
                .clone()
                .into_iter()
                .map(|path| IncludeFiles {
                    path,
                    is_default: false,
                })
                .collect::<Vec<_>>()
        };

        // We process each file and include then in the public directory
        let dest_dir = opts.profile_target_dir()?.join(&opts.public_dir);

        process_stylesheet(self.options.as_ref(), &dest_dir).await?;

        process_files(include_files, dest_dir.as_path(), opts)
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
        tracing::debug!("target dir: {}", target_dir.display());

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
        tracing::debug!("target dir: {}", target_dir.display());

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

    async fn spawn_wasm_bindgen(&self) -> anyhow::Result<Child> {
        let opts = &self.options;

        let wasm_bindgen = WasmBindgen::load().await?;
        let mut cmd = wasm_bindgen.async_cmd();

        // args
        cmd.args(["--target", "web"]).arg("--no-typescript");

        // out dir
        let mut out_dir = opts.profile_target_dir()?;
        out_dir.push(&opts.public_dir);
        tracing::debug!("wasm-bindgen out-dir: {}", out_dir.display());

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

        let mut wasm_dir = wasm_target_dir;
        let lib_name = crate::utils::get_cargo_lib_name().context("Failed to get lib name")?;
        wasm_dir.push(format!("{lib_name}.wasm"));
        tracing::debug!("wasm file dir: {}", wasm_dir.display());

        cmd.arg(wasm_dir);

        // Run
        let child = cmd.spawn()?;
        Ok(child)
    }

    async fn optimize_wasm(&self) -> anyhow::Result<()> {
        let opts = &self.options;
        let mut optimize = opts.opt_level;

        // If not set we default to the max level on release.
        if opts.release && optimize.is_none() {
            optimize = Some(WasmOptimizationLevel::Level4);
        }

        let Some(opt_level) = optimize else {
            return Ok(());
        };

        tracing::info!("Optimizing wasm... {opt_level}");
        let lib_name = crate::utils::get_cargo_lib_name().context("Failed to get lib name")?;
        let mut wasm_dir = opts.profile_target_dir()?;
        wasm_dir.push(&opts.public_dir);

        let wasm_input = wasm_dir.join(format!("{lib_name}_bg.wasm"));

        anyhow::ensure!(
            wasm_input.exists(),
            "wasm file not found at: {}",
            wasm_input.display()
        );

        // We output to itself
        let wasm_out = wasm_dir.join(format!("{lib_name}_bg_opt.wasm"));

        match opt_level {
            WasmOptimizationLevel::Size => {
                OptimizationOptions::new_optimize_for_size().run(&wasm_input, &wasm_out)?;
            }
            WasmOptimizationLevel::SizeAggressive => {
                OptimizationOptions::new_optimize_for_size_aggressively()
                    .run(&wasm_input, &wasm_out)?;
            }
            WasmOptimizationLevel::Level0 => {
                OptimizationOptions::new_opt_level_0().run(&wasm_input, &wasm_out)?;
            }
            WasmOptimizationLevel::Level1 => {
                OptimizationOptions::new_opt_level_1().run(&wasm_input, &wasm_out)?;
            }
            WasmOptimizationLevel::Level2 => {
                OptimizationOptions::new_opt_level_2().run(&wasm_input, &wasm_out)?;
            }
            WasmOptimizationLevel::Level3 => {
                OptimizationOptions::new_opt_level_3().run(&wasm_input, &wasm_out)?;
            }
            WasmOptimizationLevel::Level4 => {
                OptimizationOptions::new_opt_level_4().run(&wasm_input, &wasm_out)?;
            }
        }

        // Replace files
        tokio::fs::rename(wasm_out, wasm_input)
            .await
            .context("failed to move optimized wasm")?;

        tracing::info!("✅ Wasm optimization done");
        Ok(())
    }
}

#[tracing::instrument(level = "debug", skip(opts))]
async fn process_files(
    include_files: Vec<IncludeFiles>,
    dest_dir: &Path,
    opts: &BuildOptions,
) -> anyhow::Result<()> {
    if include_files.is_empty() {
        return Ok(());
    }

    // Get all the files to process
    let mut files = get_files_to_process(&include_files, opts)?;

    // Get the pipelines to process all the files
    let mut pipelines = pipelines();

    // The futures to await
    let mut tasks = Vec::new();

    loop {
        if files.is_empty() {
            if !pipelines.is_empty() {
                let pipeline_names = pipelines.iter().map(|p| p.name()).collect::<Vec<_>>();
                tracing::debug!(
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
                let target = files.remove(i);
                target_files.push(target);
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

    // We await all in parallel
    let results = futures::future::join_all(tasks).await;
    for ret in results {
        if let Err(err) = ret {
            tracing::error!("{err}");
        }
    }

    Ok(())
}

fn detect_stylesheet_file() -> anyhow::Result<Option<PathBuf>> {
    const STYLE_SHEET_FILES: &[&str] = &["global.css", "global.scss", "global.sass", "global.less"];
    let cwd = std::env::current_dir().context("failed to get current working directory")?;

    for file in STYLE_SHEET_FILES {
        let path = cwd.join(file);

        if path.exists() {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

#[tracing::instrument(level = "debug", skip(opts))]
async fn process_stylesheet(opts: &BuildOptions, dest_dir: &Path) -> anyhow::Result<()> {
    let detected_style_sheet = detect_stylesheet_file()?;
    let Some(style_file) = opts.styles.as_ref().or(detected_style_sheet.as_ref()) else {
        tracing::warn!("no stylesheet declared");
        return Ok(())
    };

    if !style_file.exists() {
        tracing::warn!("stylesheet file `{}` was not found", style_file.display());
        return Ok(());
    }

    tracing::debug!("using stylesheet file: {}", style_file.display());

    let Some(ext) = style_file.extension() else {
        anyhow::bail!("couldn't get extension of style file `{}`", style_file.display());
    };

    match ext.to_string_lossy().as_ref() {
        "css" => bundle_css(&style_file, dest_dir, opts)
            .await
            .with_context(|| format!("failed to bundle css: {}", style_file.display()))?,
        "sass" | "scss" | "less" => {
            bundle_sass(&style_file, dest_dir, opts)
                .await
                .with_context(|| {
                    format!(
                        "failed to bundle {}: {}",
                        ext.to_string_lossy(),
                        style_file.display()
                    )
                })?
        }
        _ => {
            anyhow::bail!("unknown stylesheet file `{}`", style_file.display());
        }
    }

    Ok(())
}

#[tracing::instrument(level = "debug")]
async fn bundle_sass(
    style_file: &Path,
    dest_dir: &Path,
    opts: &BuildOptions,
) -> anyhow::Result<()> {
    let sass = Sass::load().await?;
    let mut cmd = sass.async_cmd();

    let dest_path = dest_dir.join(style_file);
    cmd.arg(style_file).arg(&dest_path);

    if opts.quiet {
        cmd.arg("--quiet");
    }

    let mut child = cmd.spawn()?;
    let status = child.wait().await?;

    anyhow::ensure!(status.success(), "failed to run sass");
    tracing::debug!("written css in {}", dest_path.display());

    Ok(())
}

#[tracing::instrument(level = "debug")]
async fn bundle_css(style_file: &Path, dest_dir: &Path, opts: &BuildOptions) -> anyhow::Result<()> {
    let fs = FileProvider::new();
    let mut bundler = Bundler::new(&fs, None, ParserOptions::default());
    let stylesheet = bundler.bundle(&style_file).unwrap();

    let css_result = stylesheet.to_css(PrinterOptions {
        minify: opts.release,
        ..Default::default()
    })?;

    let dest_path = dest_dir.join(style_file);
    let mut file = tokio::fs::File::create(&dest_path).await?;
    let code = css_result.code;

    let mut reader = BufReader::new(code.as_bytes());
    tokio::io::copy(&mut reader, &mut file)
        .await
        .context("failed to copy css stylesheet")?;

    tracing::debug!("written css in {}", dest_path.display());

    Ok(())
}

fn get_files_to_process(
    include_files: &[IncludeFiles],
    opts: &BuildOptions,
) -> anyhow::Result<Vec<PipelineFile>> {
    let cwd = std::env::current_dir().context("failed to get current working directory")?;
    let mut files = Vec::new();

    // We iterate over each path and check if can be include in the public directory
    for include in include_files {
        let path = &include.path;

        if !path.exists() {
            if !include.is_default {
                tracing::warn!("`{}` does not exist", path.display());
            }
            continue;
        }

        // If is a file and is valid we push it to the pipeline files
        if path.is_file() {
            check_can_include(
                &cwd,
                &path,
                opts.allow_include_external,
                opts.allow_include_src,
            )?;

            // SAFETY: We already check the path
            let file = dunce::canonicalize(&path).unwrap();
            let base_dir = file
                .parent()
                .unwrap()
                .to_owned()
                .canonicalize()
                .with_context(|| format!("failed to get base dir of file: {}", file.display()))?;

            tracing::debug!("Entry: {}", path.display());

            files.push(PipelineFile { base_dir, file });
        }
        // If is a directory, we include all subdirectories files
        else if path.is_dir() {
            // We use a glob to get all the files
            let pattern = format!("{}/**/*", path.to_str().unwrap());
            for entry in glob::glob(&pattern)? {
                let entry = match entry {
                    Ok(e) => e,
                    Err(err) => {
                        tracing::debug!("{err}");
                        continue;
                    }
                };

                // We ignore subdirectories
                if entry.is_dir() {
                    continue;
                }

                check_can_include(
                    &cwd,
                    &path,
                    opts.allow_include_external,
                    opts.allow_include_src,
                )?;

                // SAFETY: the file exists
                let base_dir = dunce::canonicalize(&path).unwrap();
                let entry = dunce::canonicalize(entry).unwrap();
                tracing::debug!("Entry: {}", entry.display());

                files.push(PipelineFile {
                    base_dir,
                    file: entry,
                });
            }
        }
    }

    Ok(files)
}

fn check_can_include(
    cwd: &Path,
    file_path: &Path,
    allow_include_external: bool,
    allow_include_src: bool,
) -> anyhow::Result<()> {
    if !allow_include_external && is_outside_directory(cwd, file_path)? {
        tracing::error!("{} is outside {}", file_path.display(), cwd.display());

        anyhow::bail!(
            "Path to include cannot be outside the current directory, use `--allow-include-external` to include files outside the current directory"
        );
    }

    if !allow_include_src && is_inside_src(cwd, file_path)? {
        tracing::error!("{} is inside `src` directory", file_path.display());

        anyhow::bail!(
            "Path to include cannot be inside the src directory, use `--allow-include-src` to include files inside the src directory"
        );
    }

    Ok(())
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
        tracing::debug!("`src` directory not found");
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

// TODO: Should we just process the pipeline in order and forget about using a Box<dyn Pipeline>?
fn pipelines() -> Vec<Box<dyn Pipeline + Send>> {
    vec![
        // The last pipeline should always be the copy files
        Box::new(CopyFilesPipeline),
    ]
}
