use std::fmt::Display;

#[derive(Debug)]
pub enum RenderError {
    NoRoot,
    InvalidProps(serde_json::Error),
}

impl Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::NoRoot => write!(
                f,
                "No element was marked with 'HASHIRA_ROOT' marker, ex <body id={{HASHIRA_ROOT}}>"
            ),
            RenderError::InvalidProps(err) => {
                write!(f, "Failed to serialize the page props: {err}")
            }
        }
    }
}

impl std::error::Error for RenderError {}
