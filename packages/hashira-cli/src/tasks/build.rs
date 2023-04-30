use crate::cli::{BuildOptions, WasmOptimizationLevel, DEFAULT_INCLUDES};
use crate::emojis;
use crate::pipelines::PipelineFile;
use crate::pipelines::{copy_files::CopyFilesPipeline, Pipeline};
use crate::tools::sass::Sass;
use crate::tools::tailwindcss::TailwindCss;
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
        self.build_client().await?;
        Ok(())
    }

    /// Builds the server
    pub async fn build_server(&self) -> anyhow::Result<()> {
        // We make some checks to ensure the hashira is not ran in an invalid directory or project
        self.check_can_build()?;

        tracing::info!("{}Building server...", emojis::BUILD);
        self.cargo_build().await?;

        tracing::info!("{}Server build done!", emojis::DONE);
        Ok(())
    }

    /// Builds the client wasm bundle
    pub async fn build_client(&self) -> anyhow::Result<()> {
        // Cleanup the public dir
        self.prepare_public_dir().await?;

        // Start Wasm build
        tracing::info!("{}Building client...", emojis::BUILD);

        self.cargo_build_wasm().await?;
        self.wasm_bindgen().await?;
        self.optimize_wasm().await?; // If the optimization flag is set or in release mode
        self.build_assets().await?;

        tracing::info!("{}Client build done!", emojis::DONE);
        Ok(())
    }

    fn check_can_build(&self) -> anyhow::Result<()> {
        // FIXME: This check exists if for some reason someone wants to bypass the checks,
        // we should consider if is valid for an user ignore this checks
        if let Ok(s) = std::env::var("HASHIRA_SKIP_BUILD_CHECK") {
            if s == "1" {
                return Ok(());
            }
        }

        let cwd = std::env::current_dir()?;
        let main_path = cwd.join("src").join("main.rs");
        let lib_path = cwd.join("src").join("lib.rs");

        anyhow::ensure!(
            lib_path.exists(),
            "`src/lib.rs` was not found, Ensure you are in the correct directory."
        );

        anyhow::ensure!(
main_path.exists(),
"`src/main.rs` was not found. Ensure you are in the correct directory.

If you are trying to run a non-rust server, currently not possible with the hashira CLI, try other method instead:
    - Checkout the README.md for more information
    - Check if the project provide a Makefile.toml or other method to execute the project"
        );

        Ok(())
    }

    async fn prepare_public_dir(&self) -> anyhow::Result<()> {
        let opts = &self.options;
        let mut public_dir = opts.profile_target_dir()?;

        public_dir.push(&opts.public_dir);
        tracing::info!(
            "{}Preparing public directory: {}",
            emojis::CONSTRUCTION,
            public_dir.display()
        );

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

    pub(crate) async fn build_assets(&self) -> anyhow::Result<()> {
        tracing::info!("{}Preparing assets...", emojis::FILES);

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

        // TODO: We can sheet if the tailwind.config exists and just check a bool
        // We try to execute tailwind, if there no configuration we return `Ok(false)`
        // and continue to process the stylesheet file normally
        if self.try_build_tailwind_css().await? == false {
            process_stylesheet(self.options.as_ref(), &dest_dir).await?;
        }

        process_assets(include_files, dest_dir.as_path(), opts)
            .await
            .context("Failed to process assets")?;

        Ok(())
    }

    async fn try_build_tailwind_css(&self) -> anyhow::Result<bool> {
        // TODO: We should cache if the tailwind config exists

        // If not tailwind.config is detected we do nothing
        if let None = get_tailwind_config_path()? {
            return Ok(false);
        }

        match get_styles_file_path(self.options.as_ref())? {
            Some(style_file) => {
                if !style_file.exists() {
                    tracing::warn!("stylesheet file `{}` was not found", style_file.display());
                    return Ok(false);
                }

                tracing::debug!("tailwindcss detected");

                // SAFETY: If the style file was detected means is a valid style sheet
                let ext = style_file.extension().and_then(|s| s.to_str()).unwrap();
                if ext != "css" {
                    tracing::warn!(
                        "styles file was detected but is not a css file: {}",
                        style_file.display()
                    );
                    return Ok(false);
                }

                let tailwind = TailwindCss::load().await?;

                let file_name = style_file.file_stem().unwrap().to_string_lossy();
                let target_dir = self.options.profile_target_dir()?;
                let public_dir = &self.options.public_dir;
                let out_dir = target_dir.join(public_dir).join(format!("{file_name}.css"));

                tracing::info!("Executing TailwindCSS...");
                let mut cmd = tailwind.async_cmd();

                cmd.arg("--input") // input
                    .arg(style_file)
                    .arg("--output")
                    .arg(&out_dir);

                if self.options.release {
                    cmd.arg("--minify");
                }

                let result = cmd.output().await;

                match result {
                    Ok(output) => {
                        if !output.status.success() {
                            let err = String::from_utf8_lossy(&output.stderr);
                            tracing::error!("tailwindcss failed: {err}");
                        }

                        tracing::debug!("written tailwindcss to: {}", out_dir.display());
                    }
                    Err(err) => {
                        tracing::warn!("failed to run tailwindcss: {err}");
                    }
                };

                // We return true because we actually ran the command
                Ok(true)
            }
            None => Ok(false),
        }
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

        tracing::info!("{}Wasm optimization done", emojis::DONE);
        Ok(())
    }
}

#[tracing::instrument(level = "debug", skip(opts))]
async fn process_assets(
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

fn get_styles_file_path(opts: &BuildOptions) -> anyhow::Result<Option<PathBuf>> {
    let detected_style_sheet = detect_stylesheet_file()?;

    if let Some(styles) = opts.styles.as_ref() {
        return Ok(Some(styles.clone()));
    }

    Ok(detected_style_sheet)
}

#[tracing::instrument(level = "debug", skip(opts))]
async fn process_stylesheet(opts: &BuildOptions, dest_dir: &Path) -> anyhow::Result<()> {
    let Some(style_file) = get_styles_file_path(opts)? else {
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
        "sass" | "scss" | "less" => bundle_sass(&style_file, dest_dir, opts)
            .await
            .with_context(|| {
                format!(
                    "failed to bundle {}: {}",
                    ext.to_string_lossy(),
                    style_file.display()
                )
            })?,
        _ => {
            anyhow::bail!("unknown stylesheet file `{}`", style_file.display());
        }
    }

    Ok(())
}

#[tracing::instrument(level = "debug", skip(opts))]
async fn bundle_sass(
    style_file: &Path,
    dest_dir: &Path,
    opts: &BuildOptions,
) -> anyhow::Result<()> {
    let sass = Sass::load().await?;
    let mut cmd = sass.async_cmd();

    // Style file may be absolute, we need only the file name and extension.
    let out_file = {
        let f = style_file.file_stem().map(Path::new).unwrap();
        f.with_extension("css")
    };
    let dest_path = dest_dir.join(out_file);
    cmd.arg(style_file).arg(&dest_path).arg("--stop-on-error");

    if opts.quiet {
        cmd.arg("--quiet");
    }

    let mut child = cmd.spawn()?;
    let status = child.wait().await?;

    anyhow::ensure!(status.success(), "failed to run sass");
    tracing::debug!("written css in {}", dest_path.display());

    Ok(())
}

#[tracing::instrument(level = "debug", skip(opts))]
async fn bundle_css(style_file: &Path, dest_dir: &Path, opts: &BuildOptions) -> anyhow::Result<()> {
    let fs = FileProvider::new();
    let mut bundler = Bundler::new(&fs, None, ParserOptions::default());
    let stylesheet = bundler.bundle(style_file).unwrap();

    let css_result = stylesheet.to_css(PrinterOptions {
        minify: opts.release,
        ..Default::default()
    })?;

    // Style file may be absolute, we need only the file name and extension.
    let out_file = {
        let f = style_file.file_stem().map(Path::new).unwrap();
        f.with_extension("css")
    };
    let dest_path = dest_dir.join(out_file);
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
                path,
                opts.allow_include_external,
                opts.allow_include_src,
            )?;

            // SAFETY: We already check the path
            let file = dunce::canonicalize(path).unwrap();
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
                    path,
                    opts.allow_include_external,
                    opts.allow_include_src,
                )?;

                // SAFETY: the file exists
                let base_dir = dunce::canonicalize(path).unwrap();
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

fn get_tailwind_config_path() -> anyhow::Result<Option<PathBuf>> {
    const TAILWIND_CONFIG: &[&str] = &["tailwind.config.js", "tailwind.config.ts"];

    let cwd = std::env::current_dir()?;

    for file_name in TAILWIND_CONFIG {
        let tailwind_config = cwd.join(file_name);

        if tailwind_config.exists() {
            return Ok(Some(tailwind_config));
        }
    }

    Ok(None)
}
