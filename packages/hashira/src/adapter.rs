use crate::{app::AppService, error::BoxError};

/// Base trait for adapters.
#[async_trait::async_trait]
pub trait Adapter {
    /// Starts the server.
    async fn serve(self, app: AppService) -> Result<(), BoxError>;
}
