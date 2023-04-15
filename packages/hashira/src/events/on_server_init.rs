use crate::app::AppService;

/// A hook called when the server is initialized.
pub trait OnServerInitialize {
    /// Called on server initialization.
    fn on_initialize(&self, service: &AppService);
}

impl<F> OnServerInitialize for F
    where
        F: Fn(&AppService) + Send + Sync + 'static,
    {
        fn on_initialize(&self, service: &AppService) {
            (self)(service);
        }
    }