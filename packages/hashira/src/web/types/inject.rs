use crate::web::FromRequest;
use std::future::{ready, Ready};

use super::DataNotFoundError;

/// Extract a value from the `app_data` implements [`Clone`].
pub struct Inject<T: Clone>(pub T);

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

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::{
        app::{
            router::{PageRouter, PageRouterWrapper},
            AppData, RequestContext,
        },
        routing::{ErrorRouter, Params},
        web::{Body, Request, Inject, FromRequest},
    };

    #[tokio::test]
    async fn inject_test() {
        #[derive(Clone)]
        struct Number(u32);

        let mut app_data = AppData::default();
        app_data.insert(String::from("hello world"));
        app_data.insert(Number(65));

        let ctx = create_request_context(app_data);

        assert!(Inject::<String>::from_request(&ctx).await.is_ok());
        assert!(Inject::<Number>::from_request(&ctx).await.is_ok());
        assert!(Inject::<f64>::from_request(&ctx).await.is_err());
    }

    fn create_request_context(app_data: AppData) -> RequestContext {
        RequestContext::new(
            Arc::new(Request::new(Body::empty())),
            Arc::new(app_data),
            PageRouterWrapper::from(PageRouter::new()),
            Arc::new(ErrorRouter::new()),
            None,
            Params::default(),
        )
    }
}
