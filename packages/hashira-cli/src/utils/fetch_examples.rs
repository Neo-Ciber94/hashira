use anyhow::Context;
use futures::TryFutureExt;
use serde::Deserialize;
use std::fmt::Display;

#[derive(Deserialize)]
struct GitHubContent {
    name: String,
    path: String,
    download_url: Option<String>,
    r#type: String,
}

#[derive(Debug, Clone)]
pub struct ExampleTemplate {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
}

impl Display for ExampleTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(description) = &self.description {
            write!(f, "{:30}{description}", self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

#[tracing::instrument(level = "DEBUG")]
pub async fn list_repository_examples() -> anyhow::Result<Vec<ExampleTemplate>> {
    const GIT_EXAMPLES: &str = "https://api.github.com/repos/Neo-Ciber94/hashira/contents/examples";

    tracing::debug!("fetching examples from: {GIT_EXAMPLES}");

    // Fetch all the files in the example folder
    let github_content: Vec<GitHubContent> = octocrab::instance()
        .get(GIT_EXAMPLES, None::<&()>)
        .await
        .context("failed to fetch github content")?;

    let mut tasks = vec![];

    tracing::debug!("{} files found in example folder", github_content.len());

    // Get each directory where the examples live
    for content in github_content.into_iter().filter(|s| s.r#type == "dir") {
        tracing::debug!("fetching description for example: {}", content.name);

        let fut = tokio::spawn(async move {
            let example_folder_url = format!("{GIT_EXAMPLES}/{}", content.name);
            let github_example_content = octocrab::instance()
                .get::<Vec<GitHubContent>, _, _>(example_folder_url, None::<&()>)
                .await
                .ok();

            // Get the url of the `Cargo.toml` to get the description of the example
            let cargo_toml_download_url = github_example_content.and_then(|x| {
                x.into_iter()
                    .find(|x| x.name == "Cargo.toml")
                    .and_then(|x| x.download_url)
            });

            // Parse the cargo.toml and get the description of the example
            if let Some(example_content) = cargo_toml_download_url {
                let manifest = reqwest::get(example_content)
                    .and_then(|x| x.bytes())
                    .await
                    .ok()
                    .and_then(|bytes| cargo_toml::Manifest::from_slice(&bytes).ok());

                let Some(cargo_toml) = manifest else {
                    return ExampleTemplate {
                        name: content.name,
                        path: content.path,
                        description: None
                    }
                };

                let description = cargo_toml.package().description().map(|x| x.to_owned());
                ExampleTemplate {
                    name: content.name,
                    path: content.path,
                    description,
                }
            } else {
                // If we fail we return the example without the description
                ExampleTemplate {
                    name: content.name,
                    path: content.path,
                    description: None,
                }
            }
        });

        tasks.push(fut);
    }

    let results = futures::future::join_all(tasks).await;
    let mut examples = vec![];

    for result in results {
        if let Ok(example) = result {
            examples.push(example);
        }
    }

    tracing::debug!("{} examples were found", examples.len());
    Ok(examples)
}
