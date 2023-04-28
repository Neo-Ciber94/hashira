use crate::app::AppService;

/// A hook called when the server is initialized.
pub trait OnServerInitialize {
    /// Called on server initialization.
    fn call(&self, service: &AppService);
}

impl<F> OnServerInitialize for F
    where
        F: Fn(&AppService) + Send + Sync + 'static,
    {
        fn call(&self, service: &AppService) {
            (self)(service);
        }
    }