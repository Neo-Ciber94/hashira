use std::fmt::Display;

#[derive(Debug)]
pub enum RenderError {
    NoRoot,
    PropSerialization,
}

impl Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::NoRoot => write!(
                f,
                "No element was marked with 'HASHIRA_ROOT' marker, ex <body id={{HASHIRA_ROOT}}>"
            ),
            RenderError::PropSerialization => {
                write!(f, "Failed to serialize the page component props")
            }
        }
    }
}
