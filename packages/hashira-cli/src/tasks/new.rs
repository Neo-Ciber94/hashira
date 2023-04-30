use anyhow::Context;
use dialoguer::{console::Term, theme::ColorfulTheme, Select};

use crate::{
    cli::{NewOptions, ProjectTemplate},
    tools::{cargo_generate::CargoGenerate, Tool, ToolExt},
};

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
        let result = cmd.output().await?;

        if !result.status.success() {
            let err = String::from_utf8_lossy(&result.stderr);
            return Err(anyhow::anyhow!("Failed to create template, {err}"));
        }

        Ok(())
    }

    async fn create_example(self) -> anyhow::Result<()> {
        // TODO: Allow to navigate over all the examples available

        let cargo_generate = CargoGenerate::load().await?;
        let example = self
            .options
            .example
            .as_ref()
            .context("no example provided")?;

        let mut cmd = cargo_generate.async_cmd();

        cmd.arg("generate")
            .arg("--git")
            .arg(REPOSITORY_URL)
            .arg(example);

        // We actually don't use the name, examples are not the same as templates,
        // but the name is required
        cmd.arg("--name").arg(example);

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
        let result = cmd.output().await?;

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
