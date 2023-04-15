use crate::app::AppService;

/// A hook called on client initialization.
pub trait OnClientInitialize {
    /// Called on client initialization.
    fn on_initialize(&self, service: &AppService);
}
