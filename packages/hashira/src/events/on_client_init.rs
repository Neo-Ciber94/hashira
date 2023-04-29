use crate::app::AppService;

/// A hook called on client initialization.
pub trait OnClientInitialize {
    /// Called on client initialization.
    fn call(&self, service: AppService);
}
