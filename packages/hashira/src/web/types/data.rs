use std::{
    future::{ready, Ready},
    ops::Deref,
    sync::Arc,
};

use thiserror::Error;

use crate::{
    app::RequestContext,
    web::{Body, FromRequest},
};

/// Wraps data for the application that can be extracted with [`RequestContext::app_data`].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Data<T: ?Sized>(Arc<T>);

impl<T> Data<T> {
    /// Constructs a new `Data` wrapper.
    pub fn new(value: T) -> Self {
        Data(Arc::new(value))
    }

    /// Returns the inner `Arc`.
    pub fn into_inner(self) -> Arc<T> {
        self.0
    }
}

impl<T> Clone for Data<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> AsRef<T> for Data<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T> Deref for Data<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<Arc<T>> for Data<T> {
    fn from(value: Arc<T>) -> Self {
        Data(value)
    }
}

#[doc(hidden)]
#[derive(Debug, Error)]
#[error("`{expected}` was not found")]
pub struct DataNotFoundError {
    pub(crate) expected: &'static str,
}

impl<T> FromRequest for Data<T>
where
    T: Send + Sync + 'static,
{
    type Error = DataNotFoundError;
    type Fut = Ready<Result<Data<T>, Self::Error>>;

    fn from_request(ctx: &RequestContext, _body: &mut Body) -> Self::Fut {
        ready(
            ctx.app_data::<Data<T>>()
                .cloned()
                .ok_or_else(|| DataNotFoundError {
                    expected: std::any::type_name::<T>(),
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::app::router::{PageRouter, PageRouterWrapper};
    use crate::app::{AppData, RequestContext};
    use crate::routing::{ErrorRouter, Params};
    use crate::web::{Body, Data, FromRequest, Request};

    #[tokio::test]
    async fn data_from_request_test() {
        let mut app_data = AppData::default();
        app_data.insert(Data::new(String::from("hello world")));
        app_data.insert(Data::new(12_i32));

        let ctx = create_request_context(app_data);

        assert!(Data::<String>::from_request(&ctx, &mut Body::empty())
            .await
            .is_ok());
        assert!(Data::<i32>::from_request(&ctx, &mut Body::empty())
            .await
            .is_ok());
        assert!(Data::<f64>::from_request(&ctx, &mut Body::empty())
            .await
            .is_err());
    }

    fn create_request_context(app_data: AppData) -> RequestContext {
        RequestContext::new(
            Arc::new(Request::new(())),
            Arc::new(app_data),
            PageRouterWrapper::from(PageRouter::new()),
            Arc::new(ErrorRouter::new()),
            None,
            Params::default(),
        )
    }
}
