use clap::Args;
use enum_iterator::Sequence;
use std::{fmt::Display, path::PathBuf};

#[derive(Args, Debug, Clone)]
pub struct NewOptions {
    #[arg(long, help = "name of the project")]
    pub name: Option<String>,

    #[arg(short, long, help = "Path to create the project")]
    pub path: Option<PathBuf>,

    #[arg(long, help = "Use the actix-web template", conflicts_with_all = &[ "axum", "rocket", "deno", "example"])]
    pub actix_web: Option<bool>,

    #[arg(long, help = "Use the axum template", conflicts_with_all = &["actix-web", "rocket", "deno", "example"])]
    pub axum: Option<bool>,

    #[arg(long, help = "Use the rocket template", conflicts_with_all = &["actix-web", "axum", "deno", "example"])]
    pub rocket: Option<bool>,

    #[arg(long, help = "Use the deno template", conflicts_with_all = &["actix-web", "axum", "rocket", "example"])]
    pub deno: Option<bool>,

    #[arg(long, help = "Use one of the examples", conflicts_with_all = &["actix-web", "axum", "rocket", "deno"])]
    pub example: Option<String>,

    #[arg(
        long,
        default_value_t = false,
        help = "Allow the template to overwrite existing files in the destination "
    )]
    pub force: bool,

    #[arg(
        long,
        default_value_t = false,
        help = "Whether if not emit nothing in console"
    )]
    pub quiet: bool,
}

impl NewOptions {
    pub fn template(&self) -> Option<ProjectTemplate> {
        if self.actix_web == Some(true) {
            return Some(ProjectTemplate::ActixWeb);
        }

        if self.axum == Some(true) {
            return Some(ProjectTemplate::Axum);
        }

        if self.rocket == Some(true) {
            return Some(ProjectTemplate::Rocket);
        }

        if self.deno == Some(true) {
            return Some(ProjectTemplate::Deno);
        }

        None
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Sequence)]
pub enum ProjectTemplate {
    ActixWeb,
    Axum,
    Rocket,
    Deno,
}

impl ProjectTemplate {
    pub fn iter() -> enum_iterator::All<ProjectTemplate> {
        enum_iterator::all::<Self>()
    }

    pub fn git_path(&self) -> &'static str {
        match self {
            ProjectTemplate::ActixWeb => "templates/with-actix-web",
            ProjectTemplate::Axum => "templates/with-axum",
            ProjectTemplate::Rocket => "templates/with-rocket",
            ProjectTemplate::Deno => "templates/with-deno",
        }
    }
}

impl Display for ProjectTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectTemplate::ActixWeb => write!(f, "Actix Web"),
            ProjectTemplate::Axum => write!(f, "Axum"),
            ProjectTemplate::Rocket => write!(f, "Rocket"),
            ProjectTemplate::Deno => write!(f, "Deno"),
        }
    }
}
