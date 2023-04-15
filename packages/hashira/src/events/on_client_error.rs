use std::panic::PanicInfo;

/// A hook called when the wasm client panics.
pub trait OnClientError {
    /// Called on panics.
    fn on_error(&self, err: &PanicInfo);
}

impl<F> OnClientError for F
where
    F: Fn(&PanicInfo) + Send + Sync + 'static,
{
    fn on_error(&self, err: &PanicInfo) {
        (self)(err)
    }
}
