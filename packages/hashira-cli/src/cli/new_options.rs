use clap::Args;
use enum_iterator::Sequence;
use std::{fmt::Display, path::PathBuf, str::FromStr};

#[derive(Args, Debug, Clone)]
pub struct NewOptions {
    #[arg(long, help = "name of the project")]
    pub name: Option<String>,

    #[arg(short, long, help = "Path to create the project")]
    pub path: Option<PathBuf>,

    #[arg(long, help = "Use the actix-web template", conflicts_with_all = &[ "axum", "rocket", "deno", "warp", "tide", "example"])]
    pub actix_web: bool,

    #[arg(long, help = "Use the axum template", conflicts_with_all = &["actix_web", "rocket", "deno", "warp", "tide","example"])]
    pub axum: bool,

    #[arg(long, help = "Use the warp template", conflicts_with_all = &["actix_web", "axum", "rocket", "deno", "tide","example"])]
    pub warp: bool,

    #[arg(long, help = "Use the tide template", conflicts_with_all = &["actix_web", "axum", "rocket", "deno", "warp", "example"])]
    pub tide: bool,

    #[arg(long, help = "Use the rocket template", conflicts_with_all = &["actix_web", "axum", "deno","warp", "tide","example"])]
    pub rocket: bool,

    #[arg(long, help = "Use the deno template", conflicts_with_all = &["actix_web", "axum", "rocket", "warp", "tide","example"])]
    pub deno: bool,

    #[arg(long, help = "Use one of the examples", 
        conflicts_with_all = &["actix_web", "axum", "rocket", "warp", "tide", "deno"],
        num_args=0..=1, 
        require_equals = true,
        default_missing_value = "")]
    pub example: Option<UseExample>,

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

    #[arg(
        long,
        default_value_t = false,
        hide = true,
        help = "Use local files for the template, this is only for testing purposes"
    )]
    pub dev: bool,
}

impl NewOptions {
    pub fn template(&self) -> Option<ProjectTemplate> {
        if self.actix_web {
            return Some(ProjectTemplate::ActixWeb);
        }

        if self.axum {
            return Some(ProjectTemplate::Axum);
        }

        if self.rocket {
            return Some(ProjectTemplate::Rocket);
        }

        if self.warp {
            return Some(ProjectTemplate::Warp);
        }

        if self.tide {
            return Some(ProjectTemplate::Tide);
        }

        if self.deno {
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
    Warp,
    Tide,
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
            ProjectTemplate::Warp => "templates/with-warp",
            ProjectTemplate::Tide => "templates/with-tide",
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
            ProjectTemplate::Warp => write!(f, "Warp"),
            ProjectTemplate::Tide => write!(f, "Tide"),
            ProjectTemplate::Deno => write!(f, "Deno"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UseExample {
    Template(String),
    Select,
}

impl FromStr for UseExample {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Ok(UseExample::Select)
        } else {
            Ok(UseExample::Template(s.to_owned()))
        }
    }
}
