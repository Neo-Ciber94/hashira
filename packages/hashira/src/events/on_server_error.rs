use crate::web::Response;

// FIXME: Make async?

/// A hook called when an response error is returned.
pub trait OnServerError {
    fn call(&self, err: Response) -> Response;
}

impl<F> OnServerError for F
where
    F: Fn(Response) -> Response + 'static,
{
    fn call(&self, err: Response) -> Response {
        (self)(err)
    }
}
