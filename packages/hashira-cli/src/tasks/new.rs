use crate::{emojis, utils::list_repository_examples};
use anyhow::Context;
use dialoguer::{console::Term, theme::ColorfulTheme, Select};

use crate::{
    cli::{NewOptions, ProjectTemplate},
    tools::{cargo_generate::CargoGenerate, Tool, ToolExt},
};

// Repository of this project
const REPOSITORY_URL: &str = "https://github.com/Neo-Ciber94/hashira.git";

pub struct NewTask {
    pub options: NewOptions,
}

impl NewTask {
    pub fn new(options: NewOptions) -> Self {
        NewTask { options }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        if self.is_example() {
            self.create_example().await
        } else {
            self.create_project().await
        }
    }

    async fn create_project(self) -> anyhow::Result<()> {
        tracing::info!("{}Creating project...", emojis::CONSTRUCTION);

        let template = self.get_template()?;
        let cargo_generate = CargoGenerate::load().await?;
        let mut cmd = cargo_generate.async_cmd();

        cmd.arg("generate")
            .arg("--git")
            .arg(REPOSITORY_URL)
            .arg(template.git_path());

        if let Some(name) = self.options.name {
            cmd.arg("--name").arg(name);
        }

        if let Some(path) = self.options.path {
            cmd.arg("--destination").arg(path);
        }

        if self.options.quiet {
            cmd.arg("--silent");
        }

        if self.options.force {
            cmd.arg("--overwrite");
        }

        tracing::debug!("Running: {cmd:?}");
        let child = cmd.spawn()?;
        let result = child.wait_with_output().await?;

        if !result.status.success() {
            let err = String::from_utf8_lossy(&result.stderr);
            return Err(anyhow::anyhow!("Failed to create template, {err}"));
        }

        Ok(())
    }

    async fn create_example(self) -> anyhow::Result<()> {
        tracing::info!("{}Generating example...", emojis::CONSTRUCTION);
        // TODO: Allow to navigate over all the examples available

        let name = self.options.name.as_deref();
        let cargo_generate = CargoGenerate::load().await?;
        let use_example = self
            .options
            .example
            .as_ref()
            .context("no example provided")?;

        let example_template = match use_example {
            crate::cli::UseExample::Template(s) => s.to_owned(),
            crate::cli::UseExample::Select => {
                let templates = list_repository_examples().await?;
                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select an example")
                    .items(&templates)
                    .interact_on_opt(&Term::stderr())?;

                match selection {
                    Some(index) => templates[index].name.to_owned(),
                    None => anyhow::bail!("You must selected a example"),
                }
            }
        };

        let example = example_template.as_str();
        let mut cmd = cargo_generate.async_cmd();

        cmd.arg("generate")
            .arg("--git")
            .arg(REPOSITORY_URL)
            .arg(format!("examples/{example}"));

        cmd.arg("--name").arg(name.unwrap_or(example));
        if name.is_none() {
            tracing::warn!("`--name` was not provided, using example name instead: {example}")
        }

        if let Some(path) = self.options.path {
            cmd.arg("--destination").arg(path);
        }

        if self.options.quiet {
            cmd.arg("--silent");
        }

        if self.options.force {
            cmd.arg("--overwrite");
        }

        tracing::debug!("Running: {cmd:?}");
        let child = cmd.spawn()?;
        let result = child.wait_with_output().await?;

        if !result.status.success() {
            let err = String::from_utf8_lossy(&result.stderr);
            return Err(anyhow::anyhow!("Failed to create example, {err}"));
        }

        Ok(())
    }

    fn get_template(&self) -> anyhow::Result<ProjectTemplate> {
        match self.options.template() {
            Some(template) => Ok(template),
            None => {
                let items = ProjectTemplate::iter().collect::<Vec<_>>();
                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select a template")
                    .items(&items)
                    .interact_on_opt(&Term::stderr())?;

                match selection {
                    Some(index) => Ok(items[index]),
                    None => anyhow::bail!("You must selected a template"),
                }
            }
        }
    }

    fn is_example(&self) -> bool {
        self.options.example.is_some()
    }
}
