use thiserror::Error;
use crate::error::BoxError;

/// An error that ocurred while rendering.
#[derive(Debug, Error)]
pub enum RenderError {
    /// Not root found
    #[error(
        "No element was marked with 'HASHIRA_ROOT' marker, ex. <div id={{HASHIRA_ROOT}}></div>"
    )]
    NoRoot,

    /// Failed to parse the props
    #[error("Failed to serialize the page props: {0}")]
    InvalidProps(serde_json::Error),

    /// An error ocurred rendering one of the chunks of the html.
    #[error("Failed to render one of the chunks: {0}")]
    ChunkError(BoxError),
}
