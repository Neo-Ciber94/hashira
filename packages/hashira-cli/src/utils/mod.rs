use std::path::PathBuf;

use anyhow::Context;
use cargo_metadata::{Metadata, MetadataCommand};
use cargo_toml::Manifest;

/// Returns the current project cargo metadata.
pub fn get_cargo_metadata() -> anyhow::Result<Metadata> {
    let metadata = MetadataCommand::new()
        .exec()
        .context("Failed to get cargo metadata")?;

    Ok(metadata)
}

/// Returns the default `target_dir` for the given release mode.
pub fn get_target_dir() -> anyhow::Result<PathBuf> {
    let metadata = get_cargo_metadata()?;
    let target_dir = metadata.target_directory.as_std_path().to_path_buf();
    Ok(target_dir)
}

/// Returns the `Cargo.toml` file data.
pub fn get_cargo_toml() -> anyhow::Result<Manifest> {
    let mut dir = std::env::current_dir()?;
    dir.push("Cargo.toml");

    let manifest = Manifest::from_path(&dir)
        .with_context(|| format!("Failed to read Cargo.toml file on: {}", dir.display()))?;
    Ok(manifest)
}

/// Returns the `lib` name of the `Cargo.toml` file.
pub fn get_cargo_lib_name() -> anyhow::Result<String> {
    let cargo_toml = get_cargo_toml()?;

    if let Some(lib) = &cargo_toml.lib {
        if let Some(name) = &lib.name {
            return Ok(name.clone());
        }
    }

    log::warn!("Cargo.toml does not contains a [lib]");

    let package = cargo_toml
        .package
        .ok_or_else(|| anyhow::anyhow!("Cargo.toml does not contains a [package]"))?;

    Ok(package.name)
}
