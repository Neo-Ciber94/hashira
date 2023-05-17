use std::fmt::Display;

use anyhow::Context;
use serde::Deserialize;

#[derive(Deserialize)]
struct GitHubContent {
    name: String,
    path: String,
    r#type: String,
}

#[derive(Debug, Clone)]
pub struct ExampleTemplate {
    pub name: String,
    pub path: String,
}

impl Display for ExampleTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub async fn list_repository_examples() -> anyhow::Result<Vec<ExampleTemplate>> {
    let content: Vec<GitHubContent> = octocrab::instance()
        .get(
            "https://api.github.com/repos/Neo-Ciber94/hashira/contents/examples",
            None::<&()>,
        )
        .await
        .context("Failed to fetch github content")?;

    let examples = content
        .into_iter()
        .filter(|s| s.r#type == "dir")
        .map(|x| ExampleTemplate {
            name: x.name,
            path: x.path,
        })
        .collect();

    Ok(examples)
}
