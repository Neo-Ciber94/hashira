use std::{fmt::Display, path::PathBuf, str::FromStr};

/// The
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Environment {
    /// The application is building.
    Build,

    /// The application is in development.
    Development,

    /// The application is in production.
    Production,
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Build => write!(f, "build"),
            Environment::Development => write!(f, "development"),
            Environment::Production => write!(f, "production"),
        }
    }
}

#[derive(Debug)]
pub struct UnknownEnvironmentError(String);

impl std::error::Error for UnknownEnvironmentError {}

impl Display for UnknownEnvironmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown environment: {}", self.0)
    }
}

impl FromStr for Environment {
    type Err = UnknownEnvironmentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            _ if s.eq_ignore_ascii_case("build") => Ok(Environment::Build),
            _ if s.eq_ignore_ascii_case("dev") || s.eq_ignore_ascii_case("development") => {
                Ok(Environment::Development)
            }
            _ if s.eq_ignore_ascii_case("prod") || s.eq_ignore_ascii_case("production") => {
                Ok(Environment::Production)
            }
            _ => Err(UnknownEnvironmentError(s.to_string())),
        }
    }
}

/// Returns the target directory.
pub(crate) fn get_target_dir() -> PathBuf {
    if let Ok(path) = std::env::var(crate::consts::HASHIRA_TARGET_DIR) {
        return PathBuf::from(path);
    }

    let mut current_dir = std::env::current_exe().expect("failed to get current executable");

    // We remove the executable and let the rest of the directory
    current_dir.pop();

    current_dir
}

/// Returns the directory where the static files will be serve.
pub(crate) fn get_public_dir() -> PathBuf {
    let mut target_dir = get_target_dir();

    let path =
        std::env::var(crate::consts::HASHIRA_PUBLIC_DIR).unwrap_or_else(|_| String::from("public"));

    target_dir.push(path);
    target_dir
}
