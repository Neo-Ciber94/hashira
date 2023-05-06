use std::future::{ready, Ready};
use crate::web::FromRequest;

use super::DataNotFoundError;

/// Extract a value from the `app_data` implements [`Clone`].
pub struct Inject<T: Clone>(T);

impl<T: Clone> Inject<T> {
    /// Returns the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromRequest for Inject<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = DataNotFoundError;
    type Fut = Ready<Result<Self, Self::Error>>;

    fn from_request(ctx: &crate::app::RequestContext) -> Self::Fut {
        match ctx.app_data::<T>().cloned() {
            Some(x) => ready(Ok(Inject(x))),
            None => {
                log::warn!("`{}` was not found in app data", std::any::type_name::<T>());
                ready(Err(DataNotFoundError {
                    expected: std::any::type_name::<T>(),
                }))
            }
        }
    }
}
