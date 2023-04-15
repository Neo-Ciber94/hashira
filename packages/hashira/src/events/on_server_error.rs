use crate::app::ResponseError;

/// A hook called when an response error will be returned.
pub trait OnServerError {
    fn on_error(&self, err: &ResponseError);
}

impl<F> OnServerError for F
where
    F: Fn(&ResponseError) + Send + Sync + 'static,
{
    fn on_error(&self, err: &ResponseError) {
        (self)(err)
    }
}
